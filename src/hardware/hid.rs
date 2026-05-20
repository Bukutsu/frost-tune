// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::error::{AppError, ErrorKind, Result};
use crate::hardware::packet_builder::init_device_session;
use crate::hardware::packet_format::{ReadTiming, READ, REPORT_ID};
use crate::hardware::protocol::DeviceProtocol;
use crate::models::{Device, DeviceInfo, Filter, PEQData};
use std::sync::atomic::{AtomicU8, Ordering};

pub const MAX_FILTER_READ_ATTEMPTS: u8 = 60;
pub const MAX_GLOBAL_GAIN_ATTEMPTS: u8 = 20;
pub const MAX_WRITE_RETRIES: u8 = 3;
pub const DEFAULT_RETRY_DELAY_MS: u64 = 20;

static GLOBAL_NONCE: AtomicU8 = AtomicU8::new(1);

pub fn reset_nonce() {
    GLOBAL_NONCE.store(1, Ordering::SeqCst);
}

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

pub fn list_devices(api: &hidapi::HidApi) -> Vec<DeviceInfo> {
    let mut devices = Vec::new();
    for device in api.device_list() {
        let device_type = Device::from_vid_pid(device.vendor_id(), device.product_id());
        if device_type != Device::Unknown {
            devices.push(device_info_from_hid(device));
        }
    }
    devices
}

pub fn send_report(device: &hidapi::HidDevice, data: &[u8], report_id: u8) -> Result<()> {
    let mut buf = [0u8; 65];
    buf[0] = report_id;
    let len = data.len().min(64);
    buf[1..1 + len].copy_from_slice(&data[..len]);
    match device.write(&buf[..]) {
        Ok(_) => Ok(()),
        Err(e) => Err(AppError::new(
            ErrorKind::WriteError,
            format!("HID Write failed: {}", e),
        )),
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
    let cfg = proto.read_timing();
    init_device_session(device, proto)?;

    let num_bands = proto.num_bands();
    let mut filter_responses = vec![None; num_bands];

    let mut had_mismatch = false;

    for i in 0u8..num_bands as u8 {
        let filter_nonce = get_next_nonce();
        let request = proto.build_filter_read_request(i, filter_nonce);
        send_report(device, &request[..], proto.report_id())?;

        let response = read_single_filter_with_nonce(device, proto, &cfg, i, filter_nonce);
        if strict && response.is_none() {
            return Err(AppError::new(
                ErrorKind::ReadTimeout,
                format!("Failed to read filter {} (nonce: {})", i, filter_nonce),
            ));
        }
        if response.is_none() {
            had_mismatch = true;
        }
        filter_responses[i as usize] = response;

        delay_ms(cfg.inter_filter_ms);
    }

    validate_filter_reads(strict, had_mismatch)?;

    let global_nonce = get_next_nonce();
    let global_gain = read_global_gain(device, proto, &cfg, global_nonce)?;
    let filters = assemble_filters(num_bands, proto, filter_responses);

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
    let mut mismatches = 0;
    // Worst-case per filter: MAX_FILTER_READ_ATTEMPTS(60) × read_timeout_ms(60ms) ×
    // mismatch limit (8) ≈ 28.8s. Designed for noisy USB pipes where the device
    // may echo stale frames before delivering the correct response.
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
                        mismatches += 1;
                        if mismatches > 8 {
                            return None;
                        }
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
    send_report(device, &request[..], proto.report_id())?;
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
    Err(AppError::new(
        ErrorKind::ReadTimeout,
        "Global gain read timeout",
    ))
}

fn assemble_filters(
    num_bands: usize,
    proto: &dyn DeviceProtocol,
    responses: Vec<Option<Vec<u8>>>,
) -> Vec<Filter> {
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

    while filters.len() < num_bands {
        let idx = filters.len() as u8;
        filters.push(Filter::enabled(idx, false));
    }

    filters
}

fn validate_filter_reads(strict: bool, had_mismatch: bool) -> Result<()> {
    if strict && had_mismatch {
        return Err(AppError::new(
            ErrorKind::ReadTimeout,
            "One or more filters failed to read",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware::packet_format::{
        CMD_FLASH_EQ, CMD_GLOBAL_GAIN, CMD_PEQ_VALUES, CMD_TEMP_WRITE, CMD_VERSION, END, READ,
        WRITE,
    };
    use crate::models::FilterType;

    struct TestProtocol;

    impl DeviceProtocol for TestProtocol {
        fn report_id(&self) -> u8 {
            REPORT_ID
        }

        fn cmd_version(&self) -> u8 {
            CMD_VERSION
        }

        fn cmd_peq_values(&self) -> u8 {
            CMD_PEQ_VALUES
        }

        fn cmd_global_gain(&self) -> u8 {
            CMD_GLOBAL_GAIN
        }

        fn cmd_temp_write(&self) -> u8 {
            CMD_TEMP_WRITE
        }

        fn cmd_flash_eq(&self) -> u8 {
            CMD_FLASH_EQ
        }

        fn build_filter_read_request(&self, index: u8, nonce: u8) -> Vec<u8> {
            vec![READ, CMD_PEQ_VALUES, nonce, 0x00, index, END]
        }

        fn build_global_gain_request(&self, nonce: u8) -> Vec<u8> {
            vec![READ, CMD_GLOBAL_GAIN, nonce, END]
        }

        fn build_filter_write_packet(
            &self,
            index: u8,
            _enabled: bool,
            _freq: f64,
            _gain: f64,
            _q: f64,
            _filter_type: u8,
        ) -> Vec<u8> {
            vec![WRITE, CMD_PEQ_VALUES, 0x00, 0x00, index, END]
        }

        fn build_global_gain_write_packet(&self, gain: i8) -> Vec<u8> {
            vec![WRITE, CMD_GLOBAL_GAIN, 0x02, 0x00, gain as u8, END]
        }

        fn build_temp_write_packet(&self) -> Vec<u8> {
            vec![WRITE, CMD_TEMP_WRITE, 0x04, 0x00, 0x00, 0xFF, 0xFF, END]
        }

        fn build_flash_eq_packet(&self) -> Vec<u8> {
            vec![WRITE, CMD_FLASH_EQ, 0x01, 0x65, END]
        }

        fn parse_filter_packet(&self, data: &[u8]) -> Option<Filter> {
            if data.len() < 34 {
                return None;
            }

            Some(Filter {
                index: data[4],
                enabled: true,
                filter_type: FilterType::Peak,
                freq: 1000,
                gain: 0.0,
                q: 1.0,
            })
        }
    }

    #[test]
    fn non_strict_reads_allow_missing_filters() {
        let responses = vec![Some(vec![0; 34]), None, Some(vec![0; 34])];
        let filters = assemble_filters(3, &TestProtocol, responses);

        assert_eq!(filters.len(), 3);
        assert!(filters[0].enabled);
        assert!(!filters[1].enabled);
        assert!(filters[2].enabled);
    }

    #[test]
    fn validation_helper_rejects_missing_filters_only_in_strict_mode() {
        assert!(validate_filter_reads(false, true).is_ok());
        assert!(validate_filter_reads(true, false).is_ok());
        assert!(validate_filter_reads(true, true).is_err());
    }
}
