// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use std::sync::mpsc;
use std::time::{Duration, Instant};
use tokio::sync::oneshot;

use crate::core::{ConnectionResult, DeviceInfo, DeviceProfile, OperationResult, PushPayload};
use crate::error::{AppError, ErrorKind};
use crate::hardware::hid::{find_device_info, list_devices};
use crate::hardware::pipeline::{pull_with_retry, push_with_verify};

pub enum LocalCommand {
    Connect(Option<DeviceInfo>, oneshot::Sender<ConnectionResult>),
    Disconnect(oneshot::Sender<OperationResult>),
    Status(oneshot::Sender<LocalStatus>),
    PullPEQ(oneshot::Sender<OperationResult>),
    PushPEQ(PushPayload, oneshot::Sender<OperationResult>),
}

#[derive(Debug, Clone)]
pub struct LocalStatus {
    pub connected: bool,
    pub physically_present: bool,
    pub device: Option<DeviceInfo>,
    pub available_devices: Vec<DeviceInfo>,
    pub backend_reset: bool,
    pub generation: u64,
    pub fatal_error: Option<String>,
}

pub struct LocalWorkerState {
    pub api: Option<hidapi::HidApi>,
    pub api_retry_count: u32,
    pub last_api_retry: Option<Instant>,
    pub fatal_error: Option<String>,
    pub last_physical_check: Instant,
    pub check_interval: Duration,

    pub device: Option<hidapi::HidDevice>,
    pub device_profile: Option<&'static dyn DeviceProfile>,
    pub info: Option<DeviceInfo>,
    pub generation: u64,
}

fn exponential_backoff_elapsed(attempts: u32, last: Option<Instant>, max_secs: u64) -> bool {
    match last {
        None => true,
        Some(last) => {
            let backoff = Duration::from_secs((2u64.saturating_pow(attempts)).min(max_secs));
            Instant::now().duration_since(last) >= backoff
        }
    }
}

impl Default for LocalWorkerState {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalWorkerState {
    pub fn new() -> Self {
        Self {
            api: None,
            api_retry_count: 0,
            last_api_retry: None,
            fatal_error: None,
            last_physical_check: Instant::now(),
            check_interval: Duration::from_millis(1000),

            device: None,
            device_profile: None,
            info: None,
            generation: 0,
        }
    }

    fn ensure_api(&mut self) {
        if self.api.is_some() {
            return;
        }

        if exponential_backoff_elapsed(self.api_retry_count, self.last_api_retry, 30) {
            self.last_api_retry = Some(Instant::now());
            self.api_retry_count += 1;

            match hidapi::HidApi::new() {
                Ok(a) => {
                    log::info!("HID API initialized successfully");
                    self.api = Some(a);
                    self.fatal_error = None;
                    self.api_retry_count = 0;
                }
                Err(e) => {
                    let msg = format!("Failed to initialize HID API: {}", e);
                    log::error!("{} (attempt {})", msg, self.api_retry_count);
                    self.fatal_error = Some(msg);
                }
            }
        }
    }

    fn perform_physical_checks(&mut self) -> bool {
        self.last_physical_check = Instant::now();
        let mut backend_reset = false;

        if let Some(ref mut api_ref) = self.api {
            if let Err(e) = api_ref.refresh_devices() {
                log::warn!("Failed to refresh USB device list: {}", e);
            }

            let is_physically_connected = find_device_info(api_ref).is_some();
            if self.device.is_some() && !is_physically_connected {
                log::warn!("DAC physically disconnected (local backend)");
                self.device = None;
                self.info = None;
                self.generation = self.generation.saturating_add(1);
                backend_reset = true;
            }
        } else if self.device.is_some() {
            self.device = None;
            self.info = None;
            self.generation = self.generation.saturating_add(1);
            backend_reset = true;
        }

        backend_reset
    }

    fn process_command(&mut self, cmd: LocalCommand, backend_reset: bool) {
        match cmd {
            LocalCommand::Connect(target_device, resp) => {
                let result = self.handle_connect(target_device);
                if result.success {
                    self.generation = self.generation.saturating_add(1);
                }
                let _ = resp.send(result);
            }
            LocalCommand::Disconnect(resp) => {
                self.device = None;
                self.info = None;
                self.generation = self.generation.saturating_add(1);
                let _ = resp.send(OperationResult {
                    success: true,
                    data: None,
                    error: None,
                });
            }
            LocalCommand::Status(resp) => {
                let status = self.handle_status(backend_reset);
                let _ = resp.send(status);
            }
            LocalCommand::PullPEQ(resp) => {
                let result = self.handle_pull();
                let _ = resp.send(result);
            }
            LocalCommand::PushPEQ(payload, resp) => {
                let result = self.handle_push(payload);
                let _ = resp.send(result);
            }
        }
    }

    fn handle_connect(&mut self, target_device: Option<DeviceInfo>) -> ConnectionResult {
        let api_ref = match &mut self.api {
            Some(api) => api,
            None => {
                return ConnectionResult {
                    success: false,
                    device: target_device,
                    error: Some(AppError::new(
                        ErrorKind::Unknown,
                        self.fatal_error
                            .clone()
                            .unwrap_or_else(|| "HID API unavailable".into()),
                    )),
                };
            }
        };

        if let Err(e) = api_ref.refresh_devices() {
            log::warn!("Failed to refresh devices before connect: {}", e);
        }

        let target_vid_pid = if let Some(ref target) = target_device {
            Some((target.vendor_id, target.product_id))
        } else {
            find_device_info(api_ref).map(|dev| (dev.vendor_id(), dev.product_id()))
        };

        let (vid, pid) = match target_vid_pid {
            Some((v, p)) => (v, p),
            None => {
                return ConnectionResult {
                    success: false,
                    device: target_device,
                    error: Some(AppError::new(
                        ErrorKind::NotConnected,
                        "No supported device found",
                    )),
                };
            }
        };

        match api_ref.open(vid, pid) {
            Ok(device) => {
                if let Err(e) = device.set_blocking_mode(true) {
                    log::warn!("Failed to set blocking mode: {}", e);
                }

                let info = list_devices(api_ref)
                    .into_iter()
                    .find(|d| d.vendor_id == vid && d.product_id == pid)
                    .unwrap_or_else(|| DeviceInfo {
                        vendor_id: vid,
                        product_id: pid,
                        path: "unknown".into(),
                        manufacturer: None,
                    });

                let profile = crate::core::device::get_profile(vid, pid);

                self.device = Some(device);
                self.info = Some(info.clone());
                self.device_profile = profile;

                ConnectionResult {
                    success: true,
                    device: Some(info),
                    error: None,
                }
            }
            Err(e) => ConnectionResult {
                success: false,
                device: target_device,
                error: Some(AppError::new(
                    ErrorKind::HardwareError,
                    format!("Failed to open HID device: {}", e),
                )),
            },
        }
    }

    fn handle_status(&mut self, backend_reset: bool) -> LocalStatus {
        if let Some(ref mut api_ref) = self.api {
            let available_devices = list_devices(api_ref);
            let physically_present = !available_devices.is_empty();

            LocalStatus {
                connected: self.device.is_some(),
                physically_present,
                device: self
                    .info
                    .clone()
                    .or_else(|| available_devices.first().cloned()),
                available_devices,
                backend_reset,
                generation: self.generation,
                fatal_error: None,
            }
        } else {
            LocalStatus {
                connected: false,
                physically_present: false,
                device: None,
                available_devices: Vec::new(),
                backend_reset,
                generation: self.generation,
                fatal_error: self.fatal_error.clone(),
            }
        }
    }

    fn handle_pull(&mut self) -> OperationResult {
        let device = match &self.device {
            Some(d) => d,
            None => {
                return OperationResult {
                    success: false,
                    data: None,
                    error: Some(AppError::new(ErrorKind::NotConnected, "Not connected")),
                }
            }
        };

        let profile = match self.device_profile {
            Some(p) => p,
            None => {
                return OperationResult {
                    success: false,
                    data: None,
                    error: Some(AppError::new(
                        ErrorKind::HardwareError,
                        "Device profile not loaded",
                    )),
                }
            }
        };
        let proto = profile.protocol();

        match pull_with_retry(device, proto.as_ref(), false) {
            Ok(peq) => OperationResult {
                success: true,
                data: Some(peq),
                error: None,
            },
            Err(e) => OperationResult {
                success: false,
                data: None,
                error: Some(e),
            },
        }
    }

    fn handle_push(&mut self, payload: PushPayload) -> OperationResult {
        let device = match &self.device {
            Some(d) => d,
            None => {
                return OperationResult {
                    success: false,
                    data: None,
                    error: Some(AppError::new(ErrorKind::NotConnected, "Not connected")),
                }
            }
        };

        let profile = match self.device_profile {
            Some(p) => p,
            None => {
                return OperationResult {
                    success: false,
                    data: None,
                    error: Some(AppError::new(
                        ErrorKind::HardwareError,
                        "Device profile not loaded",
                    )),
                }
            }
        };
        let proto = profile.protocol();

        match push_with_verify(device, profile, proto.as_ref(), payload) {
            Ok(peq) => OperationResult {
                success: true,
                data: Some(peq),
                error: None,
            },
            Err(e) => OperationResult {
                success: false,
                data: None,
                error: Some(e),
            },
        }
    }
}

pub fn run_local_worker(rx: mpsc::Receiver<LocalCommand>) {
    let mut state = LocalWorkerState::new();

    loop {
        state.ensure_api();

        let now = Instant::now();
        let time_since_check = now.duration_since(state.last_physical_check);
        let mut backend_reset = false;

        if time_since_check >= state.check_interval {
            backend_reset = state.perform_physical_checks();
        }

        let timeout = state.check_interval.saturating_sub(time_since_check);

        match rx.recv_timeout(timeout) {
            Ok(cmd) => {
                state.process_command(cmd, backend_reset);
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Just let the loop continue and run physical checks next tick
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                // Channel closed, terminate thread
                break;
            }
        }
    }
}
