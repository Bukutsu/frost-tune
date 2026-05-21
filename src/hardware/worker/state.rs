// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use std::sync::mpsc;

use crate::hardware::hid::find_device_info;
use crate::hardware::worker::backend::TransportBackend;
use crate::hardware::worker::connection::worker_connect;
use crate::hardware::worker::ops::{worker_pull_peq, worker_push_peq};
use crate::hardware::worker::{BackendKind, UsbCommand, WorkerStatus};

#[cfg(target_os = "linux")]
use crate::hardware::elevated_transport::ElevatedTransport;
#[cfg(target_os = "linux")]
use crate::hardware::helper_ipc::{HelperRequest, HelperResponse};

pub(crate) enum IterationResult {
    Continue,
    Stop,
}

pub(crate) struct WorkerState {
    pub backend: Option<TransportBackend>,
    pub preferred_backend: BackendKind,
    pub api: Option<hidapi::HidApi>,
    pub api_retry_count: u32,
    pub last_api_retry: Option<std::time::Instant>,
    pub fatal_error: Option<String>,
    pub last_physical_check: std::time::Instant,
    pub generation: u64,
    pub check_interval: std::time::Duration,
    pub elevated_respawn_attempts: u32,
    pub last_elevated_respawn: Option<std::time::Instant>,
}

fn exponential_backoff_elapsed(
    attempts: u32,
    last: Option<std::time::Instant>,
    max_secs: u64,
) -> bool {
    match last {
        None => true,
        Some(last) => {
            let backoff =
                std::time::Duration::from_secs((2u64.saturating_pow(attempts)).min(max_secs));
            std::time::Instant::now().duration_since(last) >= backoff
        }
    }
}

impl WorkerState {
    pub fn new() -> Self {
        Self {
            backend: None,
            preferred_backend: BackendKind::Local,
            api: None,
            api_retry_count: 0,
            last_api_retry: None,
            fatal_error: None,
            last_physical_check: std::time::Instant::now(),
            generation: 0,
            check_interval: std::time::Duration::from_millis(1000),
            elevated_respawn_attempts: 0,
            last_elevated_respawn: None,
        }
    }

    fn ensure_api(&mut self) {
        if self.api.is_some() {
            return;
        }

        let should_retry =
            exponential_backoff_elapsed(self.api_retry_count, self.last_api_retry, 30);

        if should_retry {
            self.last_api_retry = Some(std::time::Instant::now());
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
        self.last_physical_check = std::time::Instant::now();
        let mut backend_reset = false;

        if let Some(ref mut api_ref) = self.api {
            if let Err(e) = api_ref.refresh_devices() {
                log::warn!("Failed to refresh USB device list: {}", e);
            }

            let local_physical_device = find_device_info(api_ref);
            let is_physically_connected = local_physical_device.is_some();
            let mut clear_backend = false;

            if let Some(current_backend) = self.backend.as_mut() {
                match current_backend {
                    TransportBackend::Local { .. } => {
                        if !is_physically_connected {
                            log::warn!("DAC physically disconnected (local backend)");
                            clear_backend = true;
                        }
                    }
                    #[cfg(target_os = "linux")]
                    TransportBackend::Elevated { transport, .. } => {
                        let status_result = transport.round_trip(&HelperRequest::Status);

                        let elevated_failed = match &status_result {
                            Ok(HelperResponse::Status {
                                connected,
                                physically_present,
                                ..
                            }) => !connected || !physically_present,
                            _ => true,
                        };

                        if elevated_failed {
                            let should_attempt = exponential_backoff_elapsed(
                                self.elevated_respawn_attempts,
                                self.last_elevated_respawn,
                                30,
                            );

                            if should_attempt && self.elevated_respawn_attempts < 3 {
                                self.last_elevated_respawn = Some(std::time::Instant::now());
                                self.elevated_respawn_attempts += 1;
                                log::warn!(
                                    "Elevated helper unresponsive, attempting respawn (attempt {}/3)",
                                    self.elevated_respawn_attempts
                                );

                                let old_backend = self.backend.take();
                                let device_info = match old_backend {
                                    Some(TransportBackend::Elevated { info, .. }) => Some(info),
                                    other => {
                                        self.backend = other;
                                        None
                                    }
                                };

                                if let Some(info) = device_info {
                                    match ElevatedTransport::spawn() {
                                        Ok(new_transport) => {
                                            log::info!("Elevated helper respawned successfully");
                                            self.backend = Some(TransportBackend::Elevated {
                                                transport: new_transport,
                                                info,
                                            });
                                            self.elevated_respawn_attempts = 0;
                                            self.last_elevated_respawn = None;
                                        }
                                        Err(e) => {
                                            log::error!("Failed to respawn elevated helper: {}", e);
                                            clear_backend = true;
                                        }
                                    }
                                } else {
                                    clear_backend = true;
                                }
                            } else {
                                log::warn!(
                                    "Elevated helper respawn attempts exhausted, clearing backend"
                                );
                                clear_backend = true;
                                self.elevated_respawn_attempts = 0;
                                self.last_elevated_respawn = None;
                            }
                        } else {
                            self.elevated_respawn_attempts = 0;
                            self.last_elevated_respawn = None;
                        }
                    }
                }
            }

            if clear_backend {
                self.backend = None;
                self.generation = self.generation.saturating_add(1);
                backend_reset = true;
            }
        } else if self.backend.is_some() {
            self.backend = None;
            self.generation = self.generation.saturating_add(1);
            backend_reset = true;
        }

        backend_reset
    }

    fn process_command(&mut self, cmd: UsbCommand, backend_reset: bool) {
        match cmd {
            UsbCommand::Connect(target_device, target_backend, resp) => {
                let result = if let Some(ref api_ref) = self.api {
                    let preferred = target_backend.unwrap_or(self.preferred_backend);
                    worker_connect(
                        &mut self.backend,
                        &mut self.preferred_backend,
                        api_ref,
                        Some(preferred),
                        target_device,
                    )
                } else {
                    crate::models::ConnectionResult {
                        success: false,
                        device: target_device,
                        error: Some(crate::error::AppError::new(
                            crate::error::ErrorKind::Unknown,
                            self.fatal_error
                                .clone()
                                .unwrap_or_else(|| "HID API unavailable".into()),
                        )),
                    }
                };
                if result.success {
                    self.generation = self.generation.saturating_add(1);
                }
                let _ = resp.send(result);
            }
            UsbCommand::Disconnect(resp) => {
                #[cfg(target_os = "linux")]
                if let Some(TransportBackend::Elevated { transport, .. }) = self.backend.as_mut() {
                    let _ = transport.round_trip(&HelperRequest::Disconnect);
                    transport.shutdown();
                }
                self.backend = None;
                self.generation = self.generation.saturating_add(1);
                let _ = resp.send(crate::models::OperationResult {
                    success: true,
                    data: None,
                    error: None,
                });
            }
            UsbCommand::Status(resp) => {
                let status = if let Some(ref mut api_ref) = self.api {
                    worker_status(&mut self.backend, api_ref, backend_reset, self.generation)
                } else {
                    WorkerStatus {
                        connected: false,
                        physically_present: false,
                        device: None,
                        available_devices: Vec::new(),
                        backend_reset,
                        generation: self.generation,
                        fatal_error: self.fatal_error.clone(),
                    }
                };
                let _ = resp.send(status);
            }
            UsbCommand::PullPEQ(resp) => {
                let result = worker_pull_peq(&mut self.backend);
                let _ = resp.send(result);
            }
            UsbCommand::PushPEQ(payload, resp) => {
                let result = worker_push_peq(&mut self.backend, payload);
                let _ = resp.send(result);
            }
        }
    }

    pub fn run_iteration(&mut self, rx: &mpsc::Receiver<UsbCommand>) -> IterationResult {
        self.ensure_api();

        let now = std::time::Instant::now();
        let time_since_check = now.duration_since(self.last_physical_check);
        let mut remaining_time = self.check_interval.saturating_sub(time_since_check);
        let mut backend_reset = false;

        if time_since_check >= self.check_interval {
            backend_reset = self.perform_physical_checks();
            remaining_time = self.check_interval;
        }

        match rx.recv_timeout(remaining_time.max(std::time::Duration::from_millis(1))) {
            Ok(cmd) => {
                self.process_command(cmd, backend_reset);
                IterationResult::Continue
            }
            Err(mpsc::RecvTimeoutError::Timeout) => IterationResult::Continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => IterationResult::Stop,
        }
    }
}

fn worker_status(
    backend: &mut Option<TransportBackend>,
    api: &mut hidapi::HidApi,
    backend_reset: bool,
    generation: u64,
) -> WorkerStatus {
    let available_devices = crate::hardware::hid::list_devices(api);
    let physically_present = !available_devices.is_empty();

    let mut should_clear_backend = false;

    let status = match backend.as_mut() {
        Some(TransportBackend::Local { info, .. }) => WorkerStatus {
            connected: true,
            physically_present,
            device: Some(info.clone()),
            available_devices: available_devices.clone(),
            backend_reset,
            generation,
            fatal_error: None,
        },
        #[cfg(target_os = "linux")]
        Some(TransportBackend::Elevated { transport, info }) => {
            match transport.round_trip(&HelperRequest::Status) {
                Ok(HelperResponse::Status {
                    connected,
                    physically_present,
                    device,
                }) => {
                    if !connected {
                        should_clear_backend = true;
                    }
                    WorkerStatus {
                        connected,
                        physically_present,
                        device: device.or_else(|| Some(info.clone())),
                        available_devices: available_devices.clone(),
                        backend_reset,
                        generation,
                        fatal_error: None,
                    }
                }
                Ok(_) | Err(_) => {
                    should_clear_backend = true;
                    WorkerStatus {
                        connected: false,
                        physically_present,
                        device: available_devices.first().cloned(),
                        available_devices: available_devices.clone(),
                        backend_reset: true,
                        generation,
                        fatal_error: None,
                    }
                }
            }
        }
        None => WorkerStatus {
            connected: false,
            physically_present,
            device: available_devices.first().cloned(),
            available_devices: available_devices.clone(),
            backend_reset,
            generation,
            fatal_error: None,
        },
    };

    if should_clear_backend {
        *backend = None;
    }

    status
}
