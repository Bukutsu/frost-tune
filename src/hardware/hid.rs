use crate::error::{AppError, ErrorKind, Result};
use crate::hardware::protocol::{DeviceProtocol, READ, REPORT_ID};
use crate::hardware::packet_builder::{init_device_session, NUM_FILTERS};
use crate::models::{Device, DeviceInfo, Filter, PEQData};
use std::sync::atomic::{AtomicU8, Ordering};

pub const MAX_FILTER_READ_ATTEMPTS: u8 = 60;
pub const MAX_GLOBAL_GAIN_ATTEMPTS: u8 = 20;
pub const MAX_WRITE_RETRIES: u8 = 3;
pub const DEFAULT_RETRY_DELAY_MS: u64 = 20;

static GLOBAL_NONCE: AtomicU8 = AtomicU8::new(1);

fn get_next_nonce() -> u8 {
    let mut next = GLOBAL_NONCE.fetch_add(1, Ordering::SeqCst);
    if next == 0 {
        next = GLOBAL_NONCE.fetch_add(1, Ordering::SeqCst);
    }
    next
}

pub fn delay_ms(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms));
}

pub fn device_info_from_hid(device_info: &hidapi::DeviceInfo) -> DeviceInfo {
    DeviceInfo {
        vendor_id: device_info.vendor_id(),
        product_id: device_info.product_id(),
        path: device_info.path().to_string_lossy().into(),
        manufacturer: device_info.manufacturer_string().map(|s| s.to_string()),
    }
}

pub fn find_device_info(api: &hidapi::HidApi) -> Option<hidapi::DeviceInfo> {
    for device in api.device_list() {
        let device_type = Device::from_vid_pid(device.vendor_id(), device.product_id());
        if device_type != Device::Unknown {
            return Some(device.clone());
        }
    }
    None
}

pub fn detect_device(api: &hidapi::HidApi) -> Device {
    for device in api.device_list() {
        let device_type = Device::from_vid_pid(device.vendor_id(), device.product_id());
        if device_type != Device::Unknown {
            return device_type;
        }
    }
    Device::Unknown
}

pub fn send_report(device: &hidapi::HidDevice, data: &[u8]) -> Result<()> {
    let mut buf = [0u8; 65];
    buf[0] = REPORT_ID;
    let len = data.len().min(64);
    buf[1..1 + len].copy_from_slice(&data[..len]);

    match device.write(&buf[..]) {
        Ok(_) => Ok(()),
        Err(e) => Err(AppError::new(ErrorKind::WriteError, format!("HID Write failed: {}", e))),
    }
}

#[derive(Debug, Clone)]
pub struct ReadTiming {
    pub post_version_ms: u64,
    pub filter_request_ms: u64,
    pub inter_filter_ms: u64,
    pub post_filter_read_ms: u64,
    pub post_global_gain_ms: u64,
    pub read_timeout_ms: u32,
}

impl Default for ReadTiming {
    fn default() -> Self {
        Self {
            post_version_ms: 50,
            filter_request_ms: 10,
            inter_filter_ms: 10,
            post_filter_read_ms: 40,
            post_global_gain_ms: 25,
            read_timeout_ms: 60,
        }
    }
}

pub fn flush_hid_buffer(device: &hidapi::HidDevice) {
    let mut buf = [0u8; 64];
    let mut total_drained = 0;
    while let Ok(count) = device.read_timeout(&mut buf[..], 5) {
        if count == 0 {
            break;
        }
        total_drained += 1;
        if total_drained > 64 {
            break;
        }
    }
}

pub fn pull_peq_internal(
    device: &hidapi::HidDevice,
    proto: &dyn DeviceProtocol,
    strict: bool,
) -> Result<PEQData> {
    let cfg = ReadTiming::default();
    init_device_session(device, proto)?;

    let mut filter_responses = vec![None; NUM_FILTERS as usize];

    for i in 0u8..NUM_FILTERS {
        let filter_nonce = get_next_nonce();
        let request = proto.build_filter_read_request(i, filter_nonce);
        send_report(device, &request[..])?;
        delay_ms(cfg.inter_filter_ms as u64);

        let response = read_single_filter_with_nonce(device, proto, &cfg, i, filter_nonce);
        if strict && response.is_none() {
            return Err(AppError::new(
                ErrorKind::ReadTimeout,
                format!("Failed to read filter {} (nonce: {})", i, filter_nonce),
            ));
        }
        filter_responses[i as usize] = response;
    }

    let global_nonce = get_next_nonce();
    let global_gain = read_global_gain(device, proto, &cfg, global_nonce)?;
    let filters = assemble_filters(proto, filter_responses);

    Ok(PEQData {
        filters,
        global_gain: global_gain as i8,
    })
}

fn read_single_filter_with_nonce(
    device: &hidapi::HidDevice,
    proto: &dyn DeviceProtocol,
    cfg: &ReadTiming,
    expected_index: u8,
    nonce: u8,
) -> Option<Vec<u8>> {
    let mut attempts = 0;
    while attempts < MAX_FILTER_READ_ATTEMPTS {
        let mut buf = [0u8; 64];
        match device.read_timeout(&mut buf[..], cfg.read_timeout_ms as i32) {
            Ok(count) if count > 0 => {
                let offset = if buf[0] == REPORT_ID { 1 } else { 0 };
                // Using proto.cmd_peq_values()
                if count >= 30 + offset
                    && buf[offset] == READ
                    && buf[offset + 1] == proto.cmd_peq_values()
                {
                    // For now keeping TP35Pro specific nonce/index offsets,
                    // but they could also be part of the trait if they differ.
                    let r_nonce = buf[offset + 2];
                    let r_idx = buf[offset + 4];

                    if r_nonce == nonce && r_idx == expected_index {
                        return Some(buf[offset..offset + 34].to_vec());
                    } else {
                        continue;
                    }
                }
            }
            Ok(_) => {}
            Err(_) => return None,
        }
        attempts += 1;
    }
    None
}

fn read_global_gain(
    device: &hidapi::HidDevice,
    proto: &dyn DeviceProtocol,
    cfg: &ReadTiming,
    nonce: u8,
) -> Result<u8> {
    delay_ms(cfg.post_filter_read_ms);
    let request = proto.build_global_gain_request(nonce);
    send_report(device, &request[..])?;
    delay_ms(cfg.post_global_gain_ms);

    let mut attempts = 0;
    while attempts < MAX_GLOBAL_GAIN_ATTEMPTS {
        let mut buf = [0u8; 64];
        match device.read_timeout(&mut buf[..], cfg.read_timeout_ms as i32) {
            Ok(count) if count > 0 => {
                let offset = if buf[0] == REPORT_ID { 1 } else { 0 };
                if count >= 6 + offset
                    && buf[offset] == READ
                    && buf[offset + 1] == proto.cmd_global_gain()
                {
                    // Constant offset 4 for gain value
                    return Ok(buf[offset + 4]);
                }
            }
            _ => {}
        }
        attempts += 1;
    }
    Err(AppError::new(ErrorKind::ReadTimeout, "Global gain read timeout"))
}

fn assemble_filters(proto: &dyn DeviceProtocol, responses: Vec<Option<Vec<u8>>>) -> Vec<Filter> {
    let mut filters = Vec::new();
    for (i, resp) in responses.into_iter().enumerate() {
        match resp {
            Some(r) => {
                if let Some(f) = proto.parse_filter_packet(&r) {
                    filters.push(f);
                } else {
                    log::warn!("Filter {} parse failed, using default", i);
                    filters.push(Filter::enabled(i as u8, false));
                }
            }
            None => {
                log::warn!("Filter {} read returned None, using default", i);
                filters.push(Filter::enabled(i as u8, false));
            }
        }
    }

    while filters.len() < 10 {
        let idx = filters.len() as u8;
        filters.push(Filter::enabled(idx, false));
    }

    filters
}
