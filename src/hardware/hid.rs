use crate::hardware::dsp::parse_filter_packet;
use crate::models::{PEQData, PRODUCT_ID, Filter, VENDOR_ID};
use crate::hardware::protocol::{
    CMD_PEQ_VALUES, CMD_GLOBAL_GAIN, CMD_VERSION,
    READ, END, REPORT_ID,
    OFFSET_NONCE, OFFSET_INDEX, OFFSET_GAIN_VALUE
};
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

pub fn find_device_info(api: &hidapi::HidApi) -> Option<hidapi::DeviceInfo> {
    for device in api.device_list() {
        if device.vendor_id() == VENDOR_ID && device.product_id() == PRODUCT_ID {
            return Some(device.clone());
        }
    }
    None
}

pub fn send_report(device: &hidapi::HidDevice, data: &[u8]) -> Result<(), String> {
    let mut buffer = vec![REPORT_ID];
    buffer.extend_from_slice(data);

    let mut attempts = 0;
    loop {
        match device.write(&buffer) {
            Ok(_) => return Ok(()),
            Err(e) => {
                attempts += 1;
                if attempts >= MAX_WRITE_RETRIES {
                    return Err(e.to_string());
                }
                delay_ms(DEFAULT_RETRY_DELAY_MS);
            }
        }
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

// Alias for backward compatibility if needed, but we'll move to ReadTiming
pub type TimingConfig = ReadTiming;

pub fn flush_hid_buffer(device: &hidapi::HidDevice) {
    let mut buf = [0u8; 64];
    let mut total_drained = 0;
    while let Ok(count) = device.read_timeout(&mut buf[..], 5) {
        if count == 0 { break; }
        total_drained += 1;
        if total_drained > 64 { break; } 
    }
}

pub fn pull_peq_internal(device: &hidapi::HidDevice, strict: bool) -> Result<PEQData, String> {
    pull_peq_internal_with_timing(device, &ReadTiming::default(), strict)
}

pub fn pull_peq_internal_with_timing(
    device: &hidapi::HidDevice,
    cfg: &ReadTiming,
    strict: bool,
) -> Result<PEQData, String> {
    flush_hid_buffer(device);
    
    // Version request
    send_report(device, &[READ, CMD_VERSION, END][..])?;
    delay_ms(cfg.post_version_ms);
    
    // Drain version response
    let mut drain = [0u8; 64];
    while let Ok(count) = device.read_timeout(&mut drain[..], 20) {
        if count == 0 { break; }
    }

    let mut filter_responses: Vec<Option<Vec<u8>>> = vec![None; 10];

    for i in 0u8..10 {
        if i > 0 { delay_ms(cfg.inter_filter_ms); } 
        
        let filter_nonce = get_next_nonce();
        send_report(device, &[READ, CMD_PEQ_VALUES, filter_nonce, 0x00, i, END][..])?;
        delay_ms(cfg.filter_request_ms);

        let response = read_single_filter_with_nonce(device, cfg, i, filter_nonce);
        if strict && response.is_none() {
            return Err(format!("Failed to read filter {} (nonce: {})", i, filter_nonce));
        }
        filter_responses[i as usize] = response;
    }

    let global_nonce = get_next_nonce();
    let global_gain = read_global_gain(device, cfg, global_nonce)?;
    let filters = assemble_filters(filter_responses);

    Ok(PEQData {
        filters,
        global_gain: global_gain as i8,
    })
}

fn read_single_filter_with_nonce(device: &hidapi::HidDevice, cfg: &ReadTiming, expected_index: u8, nonce: u8) -> Option<Vec<u8>> {
    let mut attempts = 0;
    while attempts < MAX_FILTER_READ_ATTEMPTS {
        let mut buf = [0u8; 64];
        match device.read_timeout(&mut buf[..], cfg.read_timeout_ms as i32) {
            Ok(count) if count > 0 => {
                let offset = if buf[0] == REPORT_ID { 1 } else { 0 };
                if count >= 30 + offset && buf[offset] == READ && buf[offset + 1] == CMD_PEQ_VALUES {
                    let r_nonce = buf[offset + OFFSET_NONCE];
                    let r_idx = buf[offset + OFFSET_INDEX];
                    
                    if r_nonce == nonce && r_idx == expected_index {
                        return Some(buf[offset..offset+34].to_vec());
                    } else {
                        // Stale packet, keep reading
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

fn read_global_gain(device: &hidapi::HidDevice, cfg: &ReadTiming, _nonce: u8) -> Result<u8, String> {
    // Use fixed nonce 0x00 for global gain - firmware may not echo nonce for this command
    delay_ms(cfg.post_filter_read_ms);
    send_report(device, &[READ, CMD_GLOBAL_GAIN, 0x00, END][..])?;
    delay_ms(cfg.post_global_gain_ms);

    let mut attempts = 0;
    while attempts < MAX_GLOBAL_GAIN_ATTEMPTS {
        let mut buf = [0u8; 64];
        match device.read_timeout(&mut buf[..], cfg.read_timeout_ms as i32) {
            Ok(count) if count > 0 => {
                let offset = if buf[0] == REPORT_ID { 1 } else { 0 };
                // Accept response without strict nonce check - firmware behavior varies
                if count >= 6 + offset && buf[offset] == READ && buf[offset + 1] == CMD_GLOBAL_GAIN {
                    return Ok(buf[offset + OFFSET_GAIN_VALUE]);
                }
            }
            _ => {}
        }
        attempts += 1;
    }
    Ok(0)
}

fn assemble_filters(responses: Vec<Option<Vec<u8>>>) -> Vec<Filter> {
    let mut filters = Vec::new();
    for (i, resp) in responses.into_iter().enumerate() {
        if let Some(r) = resp {
            if let Some(f) = parse_filter_packet(&r) {
                filters.push(f);
                continue;
            }
        }
        filters.push(Filter::enabled(i as u8, false));
    }
    filters
}
