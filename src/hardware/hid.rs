// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::core::device::protocol::DeviceProtocol;
use crate::core::device::timing::ReadTiming;
use crate::core::{DeviceInfo, Filter, PEQData};
use crate::error::{AppError, ErrorKind, Result};
use crate::hardware::packet_builder::init_device_session;

impl crate::hardware::PhysicalInterface for hidapi::HidDevice {
    fn write(&self, data: &[u8]) -> Result<usize> {
        hidapi::HidDevice::write(self, data)
            .map_err(|e| AppError::new(ErrorKind::WriteError, format!("HID Write failed: {}", e)))
    }

    fn read_timeout(&self, data: &mut [u8], timeout_ms: u32) -> Result<usize> {
        hidapi::HidDevice::read_timeout(self, data, timeout_ms as i32)
            .map_err(|e| AppError::new(ErrorKind::ReadTimeout, format!("HID Read failed: {}", e)))
    }

    fn flush(&self) -> Result<()> {
        let mut buf = [0u8; 64];
        let mut total_drained = 0;
        while let Ok(count) = hidapi::HidDevice::read_timeout(self, &mut buf[..], DRAIN_TIMEOUT_MS)
        {
            if count == 0 {
                break;
            }
            total_drained += 1;
            if total_drained > MAX_DRAIN_ITERATIONS {
                break;
            }
        }
        Ok(())
    }
}

pub struct HidDiscoveryProvider;

impl crate::hardware::DiscoveryProvider for HidDiscoveryProvider {
    fn list_devices(&self) -> Result<Vec<DeviceInfo>> {
        let api = hidapi::HidApi::new().map_err(|e| {
            AppError::new(
                ErrorKind::HardwareError,
                format!("Failed to init HID API: {}", e),
            )
        })?;
        Ok(list_devices(&api))
    }

    fn open_device(
        &self,
        info: &DeviceInfo,
    ) -> Result<Box<dyn crate::hardware::PhysicalInterface>> {
        let api = hidapi::HidApi::new().map_err(|e| {
            AppError::new(
                ErrorKind::HardwareError,
                format!("Failed to init HID API: {}", e),
            )
        })?;
        let path_str = &info.path;
        let path = std::ffi::CString::new(path_str.as_str()).map_err(|e| {
            AppError::new(
                ErrorKind::InvalidPayload,
                format!("Invalid device path: {}", e),
            )
        })?;
        let device = api.open_path(&path).map_err(|e| {
            AppError::new(
                ErrorKind::PermissionDenied,
                format!("Failed to open HID device: {}", e),
            )
        })?;
        device.set_blocking_mode(true).map_err(|e| {
            AppError::new(
                ErrorKind::HardwareError,
                format!("Failed to set blocking mode: {}", e),
            )
        })?;
        Ok(Box::new(device))
    }
}

pub const MAX_FILTER_READ_ATTEMPTS: u8 = 60;
pub const MAX_GLOBAL_GAIN_ATTEMPTS: u8 = 20;
pub const MAX_WRITE_RETRIES: u8 = 3;
pub const DEFAULT_RETRY_DELAY_MS: u64 = 20;

const DRAIN_TIMEOUT_MS: i32 = 5;
const MAX_DRAIN_ITERATIONS: usize = 64;
const MAX_MISMATCH_COUNT: u8 = 8;

/// Per-operation nonce counter. Created fresh by `init_device_session` so there
/// is no shared mutable state between concurrent (or sequential) operations.
pub struct DeviceSession {
    nonce: u8,
}

impl Default for DeviceSession {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceSession {
    pub fn new() -> Self {
        Self { nonce: 1 }
    }

    pub fn next_nonce(&mut self) -> u8 {
        loop {
            let n = self.nonce;
            self.nonce = self.nonce.wrapping_add(1);
            if n != 0 {
                return n;
            }
        }
    }
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
        product_string: device_info.product_string().map(|s| s.to_string()),
    }
}

pub fn find_device_info(api: &hidapi::HidApi) -> Option<hidapi::DeviceInfo> {
    for device in api.device_list() {
        if let Some(profile) = crate::hardware::get_profile(device.vendor_id(), device.product_id())
        {
            let info = device_info_from_hid(device);
            if profile.filter_device(&info) {
                return Some(device.clone());
            }
        }
    }
    None
}

pub fn list_devices(api: &hidapi::HidApi) -> Vec<DeviceInfo> {
    let mut devices = Vec::new();
    for device in api.device_list() {
        if let Some(profile) = crate::hardware::get_profile(device.vendor_id(), device.product_id())
        {
            let info = device_info_from_hid(device);
            if profile.filter_device(&info) {
                devices.push(info);
            }
        }
    }
    devices
}

pub fn send_report(
    device: &dyn crate::hardware::PhysicalInterface,
    data: &[u8],
    framer: &dyn crate::hardware::PacketFramer,
) -> Result<()> {
    let framed = framer.frame_packet(data);
    match device.write(&framed) {
        Ok(_) => Ok(()),
        Err(e) => Err(AppError::new(
            ErrorKind::WriteError,
            format!("HID Write failed: {}", e.message),
        )),
    }
}

pub fn flush_hid_buffer(device: &dyn crate::hardware::PhysicalInterface) {
    let _ = device.flush();
}

pub fn pull_peq_internal(
    device: &dyn crate::hardware::PhysicalInterface,
    proto: &dyn DeviceProtocol,
    strict: bool,
    num_bands: usize,
    check_in: &dyn Fn() -> bool,
) -> Result<PEQData> {
    let cfg = proto.read_timing();
    let framer_box = proto.framer();
    let framer = framer_box.as_ref();
    let mut session = init_device_session(device, proto)?;

    let mut filter_results: Vec<Option<Filter>> = Vec::with_capacity(num_bands);
    let mut had_mismatch = false;

    for i in 0u8..num_bands as u8 {
        if check_in() {
            return Err(AppError::new(ErrorKind::OperationCancelled, "Cancelled"));
        }

        let nonce = session.next_nonce();
        let request = proto.build_filter_read_request(i, nonce);
        send_report(device, &request, framer)?;

        let result = read_single_filter(device, proto, framer, &cfg, i, nonce);
        if strict && result.is_none() {
            return Err(AppError::new(
                ErrorKind::ReadTimeout,
                format!("Failed to read filter {} (nonce: {})", i, nonce),
            ));
        }
        if result.is_none() {
            had_mismatch = true;
        }
        filter_results.push(result);

        delay_ms(cfg.inter_filter_ms);
    }

    validate_filter_reads(strict, had_mismatch)?;

    let global_nonce = session.next_nonce();
    let global_gain = read_global_gain(device, proto, framer, &cfg, global_nonce)?;
    let filters = assemble_filters(num_bands, filter_results);

    Ok(PEQData {
        filters,
        global_gain,
    })
}

/// Read a single filter response from the device, matching by `index` and `nonce`.
/// The protocol decides whether an incoming packet is the response we expect.
fn read_single_filter(
    device: &dyn crate::hardware::PhysicalInterface,
    proto: &dyn DeviceProtocol,
    framer: &dyn crate::hardware::PacketFramer,
    cfg: &ReadTiming,
    expected_index: u8,
    nonce: u8,
) -> Option<Filter> {
    let mut attempts = 0;
    let mut mismatches = 0;
    while attempts < MAX_FILTER_READ_ATTEMPTS {
        let mut buf = [0u8; 64];
        match device.read_timeout(&mut buf[..], cfg.read_timeout_ms) {
            Ok(count) if count > 0 => {
                if let Ok(data) = framer.unframe_packet(&buf[..count]) {
                    if proto.matches_filter_response(&data, expected_index, nonce) {
                        return proto.parse_filter_response(&data);
                    } else if !data.is_empty() {
                        mismatches += 1;
                        if mismatches <= 2 || mismatches == MAX_MISMATCH_COUNT {
                            log::trace!(
                                "Filter read mismatch: expected index={} nonce={:#04x}, \
                                 got {:02x?} (first {} bytes)",
                                expected_index,
                                nonce,
                                &data[..data.len().min(8)],
                                data.len().min(8)
                            );
                        }
                        if mismatches > MAX_MISMATCH_COUNT {
                            return None;
                        }
                        continue;
                    }
                }
            }
            Ok(_) => {}
            Err(e) => {
                log::warn!(
                    "HID read error for filter {}: {}",
                    expected_index,
                    e.message
                );
                return None;
            }
        }
        attempts += 1;
    }
    None
}

fn read_global_gain(
    device: &dyn crate::hardware::PhysicalInterface,
    proto: &dyn DeviceProtocol,
    framer: &dyn crate::hardware::PacketFramer,
    cfg: &ReadTiming,
    nonce: u8,
) -> Result<i8> {
    delay_ms(cfg.post_filter_read_ms);
    let request = proto.build_global_gain_request(nonce);
    send_report(device, &request, framer)?;
    delay_ms(cfg.post_global_gain_ms);

    let mut attempts = 0;
    while attempts < MAX_GLOBAL_GAIN_ATTEMPTS {
        let mut buf = [0u8; 64];
        match device.read_timeout(&mut buf[..], cfg.read_timeout_ms) {
            Ok(count) if count > 0 => {
                if let Ok(data) = framer.unframe_packet(&buf[..count]) {
                    if proto.matches_global_gain_response(&data, nonce) {
                        if let Some(gain) = proto.parse_global_gain_response(&data) {
                            return Ok(gain);
                        }
                    }
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

fn assemble_filters(num_bands: usize, results: Vec<Option<Filter>>) -> Vec<Filter> {
    let mut filters: Vec<Filter> = results
        .into_iter()
        .enumerate()
        .map(|(i, opt)| match opt {
            Some(f) => f,
            None => {
                log::warn!("Filter {} read returned None, using default", i);
                Filter::enabled(i as u8, false)
            }
        })
        .collect();

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
    use crate::core::FilterType;
    use crate::hardware::devices::tp35pro::{
        CMD_FLASH_EQ, CMD_GLOBAL_GAIN, CMD_PEQ_VALUES, CMD_TEMP_WRITE, CMD_VERSION, END, READ,
        WRITE,
    };

    #[allow(dead_code)]
    struct TestProtocol;

    impl DeviceProtocol for TestProtocol {
        fn report_id(&self) -> u8 {
            0x4B
        }

        fn build_init_packets(&self) -> Vec<Vec<u8>> {
            vec![vec![READ, CMD_VERSION, END]]
        }

        fn build_filter_read_request(&self, index: u8, nonce: u8) -> Vec<u8> {
            vec![READ, CMD_PEQ_VALUES, nonce, 0x00, index, END]
        }

        fn matches_filter_response(&self, data: &[u8], index: u8, nonce: u8) -> bool {
            data.len() >= 34
                && data[0] == READ
                && data[1] == CMD_PEQ_VALUES
                && data[2] == nonce
                && data[4] == index
        }

        fn parse_filter_response(&self, data: &[u8]) -> Option<Filter> {
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

        fn build_filter_write_packet(
            &self,
            index: u8,
            _filter: &Filter,
            _dsp_sample_rate: f64,
        ) -> Vec<u8> {
            vec![WRITE, CMD_PEQ_VALUES, 0x00, 0x00, index, END]
        }

        fn build_global_gain_request(&self, nonce: u8) -> Vec<u8> {
            vec![READ, CMD_GLOBAL_GAIN, nonce, END]
        }

        fn matches_global_gain_response(&self, data: &[u8], _nonce: u8) -> bool {
            data.len() >= 6 && data[0] == READ && data[1] == CMD_GLOBAL_GAIN
        }

        fn parse_global_gain_response(&self, data: &[u8]) -> Option<i8> {
            if data.len() > 4 {
                Some(data[4] as i8)
            } else {
                None
            }
        }

        fn build_global_gain_write_packet(&self, gain: i8) -> Vec<u8> {
            vec![WRITE, CMD_GLOBAL_GAIN, 0x02, 0x00, gain as u8, END]
        }

        fn build_commit_packets(&self) -> Vec<Vec<u8>> {
            vec![
                vec![WRITE, CMD_TEMP_WRITE, 0x04, 0x00, 0x00, 0xFF, 0xFF, END],
                vec![WRITE, CMD_FLASH_EQ, 0x01, 0x65, END],
            ]
        }
    }

    #[test]
    fn non_strict_reads_allow_missing_filters() {
        let results: Vec<Option<Filter>> = vec![
            Some(Filter {
                index: 0,
                enabled: true,
                filter_type: FilterType::Peak,
                freq: 1000,
                gain: 0.0,
                q: 1.0,
            }),
            None,
            Some(Filter {
                index: 2,
                enabled: true,
                filter_type: FilterType::Peak,
                freq: 1000,
                gain: 0.0,
                q: 1.0,
            }),
        ];
        let filters = assemble_filters(3, results);

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
