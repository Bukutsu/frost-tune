pub mod backend;
pub mod connection;
pub mod ops;

use std::sync::mpsc;
use std::thread;

use crate::hardware::hid::find_device_info;
use crate::hardware::worker::backend::TransportBackend;
use crate::hardware::worker::connection::worker_connect;
use crate::hardware::worker::ops::{worker_pull_peq, worker_push_peq};
use crate::models::{ConnectionResult, DeviceInfo, OperationResult, PushPayload};
pub use backend::BackendKind;

#[cfg(target_os = "linux")]
use crate::hardware::elevated_transport::ElevatedTransport;
#[cfg(target_os = "linux")]
use crate::hardware::helper_ipc::{HelperRequest, HelperResponse};

#[derive(Debug, Clone)]
pub struct WorkerStatus {
    pub connected: bool,
    pub physically_present: bool,
    pub device: Option<DeviceInfo>,
    pub available_devices: Vec<DeviceInfo>,
    pub backend_reset: bool,
    pub generation: u64,
    pub fatal_error: Option<String>,
}

pub enum UsbCommand {
    Connect(
        Option<DeviceInfo>,
        Option<BackendKind>,
        mpsc::Sender<ConnectionResult>,
    ),
    Disconnect(mpsc::Sender<OperationResult>),
    Status(mpsc::Sender<WorkerStatus>),
    PullPEQ(mpsc::Sender<OperationResult>),
    PushPEQ(PushPayload, mpsc::Sender<OperationResult>),
}

pub struct UsbWorker {
    tx: mpsc::Sender<UsbCommand>,
}

fn panic_message(panic_info: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = panic_info.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = panic_info.downcast_ref::<String>() {
        s.clone()
    } else {
        "Unknown panic in worker thread".to_string()
    }
}

fn run_worker_iteration(
    backend: &mut Option<TransportBackend>,
    preferred_backend: &mut BackendKind,
    api: &mut Option<hidapi::HidApi>,
    api_retry_count: &mut u32,
    last_api_retry: &mut Option<std::time::Instant>,
    fatal_error: &mut Option<String>,
    last_physical_check: &mut std::time::Instant,
    generation: &mut u64,
    check_interval: std::time::Duration,
    rx: &mpsc::Receiver<UsbCommand>,
    elevated_respawn_attempts: &mut u32,
    last_elevated_respawn: &mut Option<std::time::Instant>,
) {
    if api.is_none() {
        let now = std::time::Instant::now();
        let should_retry = match *last_api_retry {
            None => true,
            Some(last) => {
                let backoff =
                    std::time::Duration::from_secs((2u64.saturating_pow(*api_retry_count)).min(30));
                now.duration_since(last) >= backoff
            }
        };

        if should_retry {
            *last_api_retry = Some(now);
            *api_retry_count += 1;
            match hidapi::HidApi::new() {
                Ok(a) => {
                    log::info!("HID API initialized successfully");
                    *api = Some(a);
                    *fatal_error = None;
                    *api_retry_count = 0;
                }
                Err(e) => {
                    let msg = format!("Failed to initialize HID API: {}", e);
                    log::error!("{} (attempt {})", msg, api_retry_count);
                    *fatal_error = Some(msg);
                }
            }
        }
    }

    let now = std::time::Instant::now();
    let time_since_check = now.duration_since(*last_physical_check);
    let mut remaining_time = check_interval.saturating_sub(time_since_check);
    let mut backend_reset = false;

    if time_since_check >= check_interval {
        *last_physical_check = now;
        remaining_time = check_interval;

        if let Some(ref mut api_ref) = api {
            if let Err(e) = api_ref.refresh_devices() {
                log::warn!("Failed to refresh USB device list: {}", e);
            }

            let local_physical_device = find_device_info(api_ref);
            let is_physically_connected = local_physical_device.is_some();

            let mut clear_backend = false;

            if let Some(current_backend) = backend.as_mut() {
                match current_backend {
                    TransportBackend::Local { .. } => {
                        if !is_physically_connected {
                            log::warn!("DAC physically disconnected (local backend)");
                            clear_backend = true;
                        }
                    }
                    #[cfg(target_os = "linux")]
                    TransportBackend::Elevated { transport, .. } => {
                        let ping_result = transport.round_trip(&HelperRequest::Ping);
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
                            let should_attempt = match *last_elevated_respawn {
                                None => true,
                                Some(last) => {
                                    let backoff = std::time::Duration::from_secs(
                                        (2u64.saturating_pow(*elevated_respawn_attempts)).min(30),
                                    );
                                    std::time::Instant::now().duration_since(last) >= backoff
                                }
                            };

                            if should_attempt && *elevated_respawn_attempts < 3 {
                                *last_elevated_respawn = Some(std::time::Instant::now());
                                *elevated_respawn_attempts += 1;
                                log::warn!(
                                    "Elevated helper unresponsive, attempting respawn (attempt {}/3)",
                                    elevated_respawn_attempts
                                );

                                let old_backend = backend.take();
                                let device_info = match old_backend {
                                    Some(TransportBackend::Elevated { info, .. }) => Some(info),
                                    other => {
                                        *backend = other;
                                        None
                                    }
                                };

                                if let Some(info) = device_info {
                                    match ElevatedTransport::spawn() {
                                        Ok(new_transport) => {
                                            log::info!(
                                                "Elevated helper respawned successfully"
                                            );
                                            *backend = Some(TransportBackend::Elevated {
                                                transport: new_transport,
                                                info,
                                            });
                                            *elevated_respawn_attempts = 0;
                                            *last_elevated_respawn = None;
                                        }
                                        Err(e) => {
                                            log::error!(
                                                "Failed to respawn elevated helper: {}",
                                                e
                                            );
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
                                *elevated_respawn_attempts = 0;
                                *last_elevated_respawn = None;
                            }
                        } else {
                            *elevated_respawn_attempts = 0;
                            *last_elevated_respawn = None;
                            if ping_result.is_err() {
                                log::warn!(
                                    "Elevated backend ping failed, may be unresponsive"
                                );
                            }
                        }
                    }
                }
            }

            if clear_backend {
                *backend = None;
                *generation = generation.saturating_add(1);
                backend_reset = true;
            }
        } else {
            if backend.is_some() {
                *backend = None;
                *generation = generation.saturating_add(1);
                backend_reset = true;
            }
        }
    }

    match rx.recv_timeout(remaining_time.max(std::time::Duration::from_millis(1))) {
        Ok(cmd) => match cmd {
            UsbCommand::Connect(target_device, target_backend, resp) => {
                let result = if let Some(ref api_ref) = api {
                    let preferred = target_backend.unwrap_or(*preferred_backend);
                    worker_connect(
                        backend,
                        preferred_backend,
                        api_ref,
                        Some(preferred),
                        target_device,
                    )
                } else {
                    ConnectionResult {
                        success: false,
                        device: target_device,
                        error: Some(crate::error::AppError::new(
                            crate::error::ErrorKind::Unknown,
                            fatal_error
                                .clone()
                                .unwrap_or_else(|| "HID API unavailable".into()),
                        )),
                    }
                };
                if result.success {
                    *generation = generation.saturating_add(1);
                }
                let _ = resp.send(result);
            }
            UsbCommand::Disconnect(resp) => {
                if let Some(current) = backend.as_mut() {
                    #[cfg(target_os = "linux")]
                    if let TransportBackend::Elevated { transport, .. } = current {
                        let _ = transport.round_trip(&HelperRequest::Disconnect);
                        transport.shutdown();
                    }
                }
                *backend = None;
                *generation = generation.saturating_add(1);
                let _ = resp.send(OperationResult {
                    success: true,
                    data: None,
                    error: None,
                });
            }
            UsbCommand::Status(resp) => {
                let status = if let Some(ref mut api_ref) = api {
                    worker_status(backend, api_ref, backend_reset, *generation)
                } else {
                    WorkerStatus {
                        connected: false,
                        physically_present: false,
                        device: None,
                        available_devices: Vec::new(),
                        backend_reset,
                        generation: *generation,
                        fatal_error: fatal_error.clone(),
                    }
                };
                let _ = resp.send(status);
            }
            UsbCommand::PullPEQ(resp) => {
                let result = worker_pull_peq(backend);
                let _ = resp.send(result);
            }
            UsbCommand::PushPEQ(payload, resp) => {
                let result = worker_push_peq(backend, payload);
                let _ = resp.send(result);
            }
        },
        Err(mpsc::RecvTimeoutError::Timeout) => {}
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            // Signal break to the caller via a sentinel; handled in the loop
            *generation = u64::MAX;
        }
    }
}

impl UsbWorker {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let mut backend: Option<TransportBackend> = None;
            let mut preferred_backend: BackendKind = BackendKind::Local;

            let mut api: Option<hidapi::HidApi> = None;
            let mut api_retry_count: u32 = 0;
            let mut last_api_retry: Option<std::time::Instant> = None;
            let mut fatal_error: Option<String> = None;

            let mut last_physical_check = std::time::Instant::now();
            let check_interval = std::time::Duration::from_millis(1000);
            let mut generation: u64 = 0;
            let mut elevated_respawn_attempts: u32 = 0;
            let mut last_elevated_respawn: Option<std::time::Instant> = None;

            loop {
                let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    run_worker_iteration(
                        &mut backend,
                        &mut preferred_backend,
                        &mut api,
                        &mut api_retry_count,
                        &mut last_api_retry,
                        &mut fatal_error,
                        &mut last_physical_check,
                        &mut generation,
                        check_interval,
                        &rx,
                        &mut elevated_respawn_attempts,
                        &mut last_elevated_respawn,
                    );
                }));

                match panic_result {
                    Ok(()) => {
                        if generation == u64::MAX {
                            break;
                        }
                    }
                    Err(panic_info) => {
                        let msg = panic_message(&panic_info);
                        log::error!("Worker thread panicked: {}", msg);
                        fatal_error = Some(msg);
                        backend = None;
                        generation = generation.saturating_add(1);
                    }
                }
            }
        });

        UsbWorker { tx }
    }

    pub fn connect(
        &self,
        device: Option<DeviceInfo>,
        backend: Option<BackendKind>,
    ) -> mpsc::Receiver<ConnectionResult> {
        let (tx, rx) = mpsc::channel();
        let _ = self.tx.send(UsbCommand::Connect(device, backend, tx));
        rx
    }

    pub fn disconnect(&self) -> mpsc::Receiver<OperationResult> {
        let (tx, rx) = mpsc::channel();
        let _ = self.tx.send(UsbCommand::Disconnect(tx));
        rx
    }

    pub fn status(&self) -> mpsc::Receiver<WorkerStatus> {
        let (tx, rx) = mpsc::channel();
        let _ = self.tx.send(UsbCommand::Status(tx));
        rx
    }

    pub fn pull_peq(&self) -> mpsc::Receiver<OperationResult> {
        let (tx, rx) = mpsc::channel();
        let _ = self.tx.send(UsbCommand::PullPEQ(tx));
        rx
    }

    pub fn push_peq(&self, payload: PushPayload) -> mpsc::Receiver<OperationResult> {
        let (tx, rx) = mpsc::channel();
        let _ = self.tx.send(UsbCommand::PushPEQ(payload, tx));
        rx
    }
}

impl Default for UsbWorker {
    fn default() -> Self {
        Self::new()
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
