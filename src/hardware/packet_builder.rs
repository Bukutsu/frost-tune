use hidapi::HidDevice;
use crate::dsp::{compute_iir_filter, convert_to_byte_array};
use crate::hid::{delay_ms, send_report};
use crate::protocol::{
    CMD_PEQ_VALUES, CMD_GLOBAL_GAIN, CMD_TEMP_WRITE, CMD_FLASH_EQ, CMD_VERSION,
    WRITE, END, READ
};
use crate::models::Filter;
use crate::error::AppError;

pub const FILTER_SLOT: u8 = 101;
pub const NUM_FILTERS: u8 = 10;

#[derive(Debug, Clone)]
pub struct WriteTiming {
    pub per_filter_ms: u64,
    pub batch_ms: u64,
    pub global_gain_ms: u64,
    pub commit_ms: u64,
}

impl Default for WriteTiming {
    fn default() -> Self {
        Self {
            per_filter_ms: 80,
            batch_ms: 100,
            global_gain_ms: 50,
            commit_ms: 500,
        }
    }
}

pub fn build_filter_packet(
    filter_index: u8,
    enabled: bool,
    mut freq: f64,
    mut gain: f64,
    mut q: f64,
    filter_type: u8,
) -> Vec<u8> {
    if !enabled {
        freq = 0.0;
        gain = 0.0;
        q = 1.0;
    }

    let b_arr = compute_iir_filter(freq, gain, q);

    let mut packet = vec![WRITE, CMD_PEQ_VALUES, 0x18, 0x00, filter_index, 0x00, 0x00];
    packet.extend_from_slice(&b_arr);
    packet.extend_from_slice(&convert_to_byte_array(freq.round() as i32, 2));
    packet.extend_from_slice(&convert_to_byte_array((q * 256.0).round() as i32, 2));
    packet.extend_from_slice(&convert_to_byte_array((gain * 256.0).round() as i32, 2));
    packet.push(filter_type);
    packet.push(0x00);
    packet.push(FILTER_SLOT);
    packet.push(END);

    packet
}

pub fn build_global_gain_packet(global_gain: i8) -> Vec<u8> {
    vec![WRITE, CMD_GLOBAL_GAIN, 0x02, 0x00, global_gain as u8, END]
}

pub fn build_temp_write_packet() -> Vec<u8> {
    vec![WRITE, CMD_TEMP_WRITE, 0x04, 0x00, 0x00, 0xFF, 0xFF, END]
}

pub fn build_flash_eq_packet() -> Vec<u8> {
    vec![WRITE, CMD_FLASH_EQ, 0x01, FILTER_SLOT, END]
}

pub fn init_device_session(device: &HidDevice) -> Result<(), AppError> {
    send_report(device, &[READ, CMD_VERSION, END][..])?;
    delay_ms(50);
    let mut drain = [0u8; 64];
    while let Ok(count) = device.read_timeout(&mut drain[..], 20) {
        if count == 0 { break; }
    }
    Ok(())
}

pub fn write_filters_and_gain(
    device: &HidDevice,
    filters: &[Filter],
    global_gain: i8,
    timing: &WriteTiming,
) -> Result<(), AppError> {
    for i in 0u8..NUM_FILTERS {
        let filter = &filters[i as usize];
        let packet = build_filter_packet(
            i, filter.enabled, filter.freq as f64, filter.gain, filter.q, filter.filter_type.into()
        );
        send_report(device, &packet[..])?;
        delay_ms(timing.per_filter_ms);
    }

    delay_ms(timing.batch_ms);

    let gain_packet = build_global_gain_packet(global_gain);
    send_report(device, &gain_packet[..])?;
    delay_ms(timing.global_gain_ms);

    Ok(())
}

pub fn commit_changes(device: &HidDevice, timing: &WriteTiming) -> Result<(), AppError> {
    let temp_packet = build_temp_write_packet();
    send_report(device, &temp_packet[..])?;
    delay_ms(timing.commit_ms);

    let flash_packet = build_flash_eq_packet();
    send_report(device, &flash_packet[..])?;

    Ok(())
}
