use std::cell::RefCell;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use tokio::sync::oneshot;

use crate::core::DeviceInfo;
use crate::error::{AppError, ErrorKind};
use crate::hardware::device_io::{DiscoveryProvider, PhysicalInterface};
use crate::hardware::pipeline::{pull_with_retry, push_with_verify, reset_with_verify};
use crate::hardware::{get_profile, DeviceProfile};
use crate::hardware::{ConnectionResult, OperationResult, PushPayload};

pub enum LocalCommand {
    Connect(Option<DeviceInfo>, oneshot::Sender<ConnectionResult>),
    Disconnect(oneshot::Sender<OperationResult>),
    Status(oneshot::Sender<LocalStatus>),
    PullPEQ(oneshot::Sender<OperationResult>),
    PushPEQ(PushPayload, bool, oneshot::Sender<OperationResult>),
    ResetPEQ(oneshot::Sender<OperationResult>),
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
    pub discovery_providers: Vec<Box<dyn DiscoveryProvider>>,
    pub fatal_error: Option<String>,
    pub last_physical_check: Instant,
    pub check_interval: Duration,

    pub device: Option<Box<dyn PhysicalInterface>>,
    pub device_profile: Option<&'static dyn DeviceProfile>,
    pub info: Option<DeviceInfo>,
    pub generation: u64,

    pub rx: RefCell<mpsc::Receiver<LocalCommand>>,
    pub pending_command: RefCell<Option<LocalCommand>>,
}

impl LocalWorkerState {
    pub fn new(rx: mpsc::Receiver<LocalCommand>) -> Self {
        Self {
            discovery_providers: vec![Box::new(crate::hardware::hid::HidDiscoveryProvider)],
            fatal_error: None,
            last_physical_check: Instant::now(),
            check_interval: Duration::from_millis(1000),

            device: None,
            device_profile: None,
            info: None,
            generation: 0,

            rx: RefCell::new(rx),
            pending_command: RefCell::new(None),
        }
    }

    /// Checks if a new command has arrived that should interrupt the current operation.
    /// Returns true if the operation should be cancelled.
    pub fn should_cancel(&self) -> bool {
        if self.pending_command.borrow().is_some() {
            return true;
        }

        match self.rx.borrow_mut().try_recv() {
            Ok(cmd) => {
                *self.pending_command.borrow_mut() = Some(cmd);
                true
            }
            Err(mpsc::TryRecvError::Empty) => false,
            Err(mpsc::TryRecvError::Disconnected) => true,
        }
    }

    fn perform_physical_checks(&mut self) -> bool {
        self.last_physical_check = Instant::now();
        let mut backend_reset = false;

        let mut physically_connected = false;
        for provider in &self.discovery_providers {
            if let Ok(devices) = provider.list_devices() {
                if let Some(ref current_info) = self.info {
                    if devices.iter().any(|d| d.path == current_info.path) {
                        physically_connected = true;
                        break;
                    }
                }
            }
        }

        if self.device.is_some() && !physically_connected {
            log::warn!("DAC physically disconnected (local backend)");
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
            LocalCommand::PushPEQ(payload, skip_verify, resp) => {
                let result = self.handle_push(payload, skip_verify);
                let _ = resp.send(result);
            }
            LocalCommand::ResetPEQ(resp) => {
                let result = self.handle_reset();
                let _ = resp.send(result);
            }
        }
    }

    fn handle_connect(&mut self, target_device: Option<DeviceInfo>) -> ConnectionResult {
        let resolved_target = if let Some(target) = target_device {
            Some(target)
        } else {
            let mut first_discovered = None;
            for provider in &self.discovery_providers {
                if let Ok(devices) = provider.list_devices() {
                    if let Some(first) = devices.first() {
                        first_discovered = Some(first.clone());
                        break;
                    }
                }
            }
            first_discovered
        };

        let target = match resolved_target {
            Some(t) => t,
            None => {
                return ConnectionResult {
                    success: false,
                    device: None,
                    error: Some(AppError::new(
                        ErrorKind::NotConnected,
                        "No supported device found",
                    )),
                };
            }
        };

        let profile = get_profile(target.vendor_id, target.product_id);
        if profile.is_none() {
            return ConnectionResult {
                success: false,
                device: Some(target),
                error: Some(AppError::new(
                    ErrorKind::HardwareError,
                    "Unsupported DAC device",
                )),
            };
        }

        for provider in &self.discovery_providers {
            if let Ok(devices) = provider.list_devices() {
                if devices.iter().any(|d| d.path == target.path) {
                    match provider.open_device(&target) {
                        Ok(opened_dev) => {
                            self.device = Some(opened_dev);
                            self.info = Some(target.clone());
                            self.device_profile = profile;

                            return ConnectionResult {
                                success: true,
                                device: Some(target),
                                error: None,
                            };
                        }
                        Err(e) => {
                            return ConnectionResult {
                                success: false,
                                device: Some(target),
                                error: Some(e),
                            };
                        }
                    }
                }
            }
        }

        ConnectionResult {
            success: false,
            device: Some(target),
            error: Some(AppError::new(
                ErrorKind::NotConnected,
                "Device not found or failed to open",
            )),
        }
    }

    fn handle_status(&mut self, backend_reset: bool) -> LocalStatus {
        let mut available_devices = Vec::new();
        for provider in &self.discovery_providers {
            if let Ok(mut devs) = provider.list_devices() {
                available_devices.append(&mut devs);
            }
        }

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
            fatal_error: self.fatal_error.clone(),
        }
    }

    fn handle_pull(&mut self) -> OperationResult {
        let check_in = || self.should_cancel();

        let device = match &self.device {
            Some(d) => d.as_ref(),
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
        let num_bands = profile.capabilities().num_bands;

        match pull_with_retry(device, proto.as_ref(), false, num_bands, &check_in) {
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

    fn handle_push(&mut self, payload: PushPayload, skip_verify: bool) -> OperationResult {
        let check_in = || self.should_cancel();

        let device = match &self.device {
            Some(d) => d.as_ref(),
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

        match push_with_verify(
            device,
            profile,
            proto.as_ref(),
            payload,
            skip_verify,
            &check_in,
        ) {
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

    fn handle_reset(&mut self) -> OperationResult {
        let check_in = || self.should_cancel();

        let device = match &self.device {
            Some(d) => d.as_ref(),
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

        match reset_with_verify(device, profile, proto.as_ref(), &check_in) {
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
    let mut state = LocalWorkerState::new(rx);

    loop {
        let now = Instant::now();
        let time_since_check = now.duration_since(state.last_physical_check);
        let mut backend_reset = false;

        if time_since_check >= state.check_interval {
            backend_reset = state.perform_physical_checks();
        }

        let pending = state.pending_command.borrow_mut().take();
        if let Some(cmd) = pending {
            state.process_command(cmd, backend_reset);
            continue;
        }

        let timeout = state.check_interval.saturating_sub(time_since_check);

        let recv_result = state.rx.borrow_mut().recv_timeout(timeout);
        match recv_result {
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
