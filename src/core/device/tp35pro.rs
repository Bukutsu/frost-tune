// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

//! EPZ TP35 Pro hardware protocol: USB packet constants, IIR math, and DeviceProtocol impl.
//!
//! All wire-level constants are TP35Pro-specific and live here rather than in a shared
//! packet_format module, so each future device can define its own wire format independently.

use crate::core::device::capabilities::{DeviceCapabilities, FilterTypeFlags};
use crate::core::device::protocol::DeviceProtocol;
use crate::core::device::registry::DeviceProfile;
use crate::core::device::timing::WriteTiming;
use crate::core::eq::{Filter, FilterType};
pub use crate::hardware::dsp::iir_math::{
    compute_iir_filter, convert_to_2byte_array, BYTE_BIT_SHIFT, GAIN_FLOAT_TO_U16_DIVISOR,
    GAIN_I16_THRESHOLD, Q_FLOAT_TO_U16_DIVISOR, U16_WRAP_AROUND,
};

// ─── Wire constants ───────────────────────────────────────────────────────────

pub const REPORT_ID: u8 = 0x4B;

pub const CMD_FLASH_EQ: u8 = 0x01;
pub const CMD_GLOBAL_GAIN: u8 = 0x03;
pub const CMD_PEQ_VALUES: u8 = 0x09;
pub const CMD_TEMP_WRITE: u8 = 0x0A;
pub const CMD_VERSION: u8 = 0x0C;

pub const READ: u8 = 0x80;
pub const WRITE: u8 = 0x01;
pub const END: u8 = 0x00;

pub const CONST_TEMP_WRITE_MAGIC_A: u8 = 0xFF;
pub const CONST_TEMP_WRITE_MAGIC_B: u8 = 0xFF;
pub const CONST_PEQ_PAYLOAD_LEN: u8 = 0x18;
pub const CONST_GLOBAL_GAIN_LEN: u8 = 0x02;
pub const CONST_TEMP_WRITE_LEN: u8 = 0x04;
pub const CONST_FLASH_EQ_LEN: u8 = 0x01;

pub const FILTER_SLOT: u8 = 101;

pub const OFFSET_CMD_TYPE: usize = 0;
pub const OFFSET_CMD: usize = 1;
pub const OFFSET_NONCE: usize = 2;
pub const OFFSET_INDEX: usize = 4;
pub const OFFSET_BIQUAD_START: usize = 7;
pub const OFFSET_FREQ_L: usize = 27;
pub const OFFSET_FREQ_H: usize = 28;
pub const OFFSET_Q_L: usize = 29;
pub const OFFSET_Q_H: usize = 30;
pub const OFFSET_GAIN_L: usize = 31;
pub const OFFSET_GAIN_H: usize = 32;
pub const OFFSET_FILTER_TYPE: usize = 33;
pub const OFFSET_SLOT: usize = 35;
pub const OFFSET_GAIN_VALUE: usize = 4;

// Minimum response length for a valid filter packet (offsets go up to 33).
const FILTER_RESPONSE_MIN_LEN: usize = 34;
// Minimum response length for a valid global gain packet (offset 4 for gain byte).
const GLOBAL_GAIN_RESPONSE_MIN_LEN: usize = 6;

// DSP IIR math is re-exported from crate::hardware::dsp::iir_math

/// Parses a TP35 Pro filter response payload (report-ID byte already stripped).
pub fn parse_filter_packet(packet: &[u8]) -> Option<Filter> {
    if packet.len() < FILTER_RESPONSE_MIN_LEN {
        return None;
    }

    let filter_index = packet[OFFSET_INDEX];
    let freq = (packet[OFFSET_FREQ_L] as u16) | ((packet[OFFSET_FREQ_H] as u16) << BYTE_BIT_SHIFT);
    let q_raw = (packet[OFFSET_Q_L] as u16) | ((packet[OFFSET_Q_H] as u16) << BYTE_BIT_SHIFT);
    let gain_raw =
        (packet[OFFSET_GAIN_L] as u16) | ((packet[OFFSET_GAIN_H] as u16) << BYTE_BIT_SHIFT);

    let gain_from_device = if gain_raw > GAIN_I16_THRESHOLD as u16 {
        (gain_raw as i32 - U16_WRAP_AROUND) as i16
    } else {
        gain_raw as i16
    };

    let q = (((q_raw as f64) / Q_FLOAT_TO_U16_DIVISOR * 100.0).round() / 100.0).max(0.01);
    let gain = ((gain_from_device as f64) / GAIN_FLOAT_TO_U16_DIVISOR * 100.0).round() / 100.0;
    let filter_type = FilterType::from(packet[OFFSET_FILTER_TYPE]);
    let enabled = !(freq == 0 && gain_from_device == 0);

    Some(Filter {
        index: filter_index,
        enabled,
        freq,
        gain,
        q,
        filter_type,
    })
}

// ─── Protocol implementation ─────────────────────────────────────────────────

pub struct TP35ProProtocol;

impl DeviceProtocol for TP35ProProtocol {
    fn report_id(&self) -> u8 {
        REPORT_ID
    }

    fn write_timing(&self) -> WriteTiming {
        WriteTiming {
            commit_step_ms: 500,
            ..WriteTiming::default()
        }
    }

    fn build_init_packets(&self) -> Vec<Vec<u8>> {
        vec![vec![READ, CMD_VERSION, END]]
    }

    fn build_filter_read_request(&self, index: u8, nonce: u8) -> Vec<u8> {
        vec![READ, CMD_PEQ_VALUES, nonce, 0x00, index, END]
    }

    fn matches_filter_response(&self, data: &[u8], index: u8, nonce: u8) -> bool {
        data.len() >= FILTER_RESPONSE_MIN_LEN
            && data[OFFSET_CMD_TYPE] == READ
            && data[OFFSET_CMD] == CMD_PEQ_VALUES
            && data[OFFSET_NONCE] == nonce
            && data[OFFSET_INDEX] == index
    }

    fn parse_filter_response(&self, data: &[u8]) -> Option<Filter> {
        parse_filter_packet(data)
    }

    fn build_filter_write_packet(&self, index: u8, filter: &Filter) -> Vec<u8> {
        let b_arr = compute_iir_filter(
            filter.filter_type,
            filter.freq as f64,
            filter.gain,
            filter.q,
        );
        let filter_type_byte: u8 = filter.filter_type.into();

        let mut packet = Vec::with_capacity(37);
        packet.extend_from_slice(&[
            WRITE,
            CMD_PEQ_VALUES,
            CONST_PEQ_PAYLOAD_LEN,
            0x00,
            index,
            0x00,
            0x00,
        ]);
        packet.extend_from_slice(&b_arr);
        packet.extend_from_slice(&convert_to_2byte_array(filter.freq as i32));
        packet.extend_from_slice(&convert_to_2byte_array((filter.q * 256.0).round() as i32));
        packet.extend_from_slice(&convert_to_2byte_array((filter.gain * 256.0).round() as i32));
        packet.extend_from_slice(&[filter_type_byte, 0x00, FILTER_SLOT, END]);

        packet
    }

    fn build_global_gain_request(&self, _nonce: u8) -> Vec<u8> {
        // TP35 Pro ignores nonce for global gain reads.
        vec![READ, CMD_GLOBAL_GAIN, 0x00, END]
    }

    fn matches_global_gain_response(&self, data: &[u8], _nonce: u8) -> bool {
        data.len() >= GLOBAL_GAIN_RESPONSE_MIN_LEN
            && data[OFFSET_CMD_TYPE] == READ
            && data[OFFSET_CMD] == CMD_GLOBAL_GAIN
    }

    fn parse_global_gain_response(&self, data: &[u8]) -> Option<i8> {
        if data.len() > OFFSET_GAIN_VALUE {
            Some(data[OFFSET_GAIN_VALUE] as i8)
        } else {
            None
        }
    }

    fn build_global_gain_write_packet(&self, gain: i8) -> Vec<u8> {
        vec![
            WRITE,
            CMD_GLOBAL_GAIN,
            CONST_GLOBAL_GAIN_LEN,
            0x00,
            gain as u8,
            END,
        ]
    }

    fn build_commit_packets(&self) -> Vec<Vec<u8>> {
        // TP35 Pro commit sequence: temp-write then flash-eq with 500 ms
        // between them (configured via write_timing().commit_step_ms).
        vec![
            vec![
                WRITE,
                CMD_TEMP_WRITE,
                CONST_TEMP_WRITE_LEN,
                0x00,
                0x00,
                CONST_TEMP_WRITE_MAGIC_A,
                CONST_TEMP_WRITE_MAGIC_B,
                END,
            ],
            vec![WRITE, CMD_FLASH_EQ, CONST_FLASH_EQ_LEN, FILTER_SLOT, END],
        ]
    }
}

pub struct TP35ProProfile;

impl DeviceProfile for TP35ProProfile {
    fn name(&self) -> &'static str {
        "EPZ TP35 Pro"
    }

    fn vendor_id(&self) -> u16 {
        0x3302
    }

    fn product_id(&self) -> u16 {
        0x43E6
    }

    fn capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities {
            num_bands: 10,
            global_gain_range: (-16, 6),
            band_gain_range: (-10.0, 10.0),
            freq_range: (20, 20000),
            q_range: (0.1, 10.0),
            supported_filter_types: FilterTypeFlags::PEAK
                | FilterTypeFlags::LOW_SHELF
                | FilterTypeFlags::HIGH_SHELF
                | FilterTypeFlags::LOW_PASS
                | FilterTypeFlags::HIGH_PASS,
            supports_per_band_enable: false,
        }
    }

    fn protocol(&self) -> Box<dyn DeviceProtocol> {
        Box::new(TP35ProProtocol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::eq::FilterType;

    fn make_filter(index: u8, freq: u16, gain: f64, q: f64) -> Filter {
        Filter {
            index,
            enabled: true,
            filter_type: FilterType::Peak,
            freq,
            gain,
            q,
        }
    }

    #[test]
    fn build_filter_write_packet_structure() {
        let proto = TP35ProProtocol;
        let filter = make_filter(0, 1000, 5.0, 1.0);
        let packet = proto.build_filter_write_packet(0, &filter);
        assert_eq!(packet[OFFSET_CMD_TYPE], WRITE);
        assert_eq!(packet[OFFSET_CMD], CMD_PEQ_VALUES);
        assert_eq!(packet[OFFSET_INDEX], 0);
        assert_eq!(packet.len(), 37);
    }

    #[test]
    fn build_global_gain_write_packet_structure() {
        let proto = TP35ProProtocol;
        let packet = proto.build_global_gain_write_packet(5);
        assert_eq!(packet[OFFSET_CMD_TYPE], WRITE);
        assert_eq!(packet[OFFSET_CMD], CMD_GLOBAL_GAIN);
        assert_eq!(packet[OFFSET_GAIN_VALUE], 5);
    }

    #[test]
    fn build_global_gain_write_packet_negative() {
        let proto = TP35ProProtocol;
        let packet = proto.build_global_gain_write_packet(-3);
        assert_eq!(packet[OFFSET_GAIN_VALUE] as i8, -3);
    }

    #[test]
    fn build_commit_packets_has_two_steps() {
        let proto = TP35ProProtocol;
        let packets = proto.build_commit_packets();
        assert_eq!(packets.len(), 2);
        assert_eq!(packets[0][1], CMD_TEMP_WRITE);
        assert_eq!(packets[1][1], CMD_FLASH_EQ);
    }

    #[test]
    fn write_timing_uses_500ms_commit_step() {
        let proto = TP35ProProtocol;
        let timing = proto.write_timing();
        assert_eq!(timing.commit_step_ms, 500);
    }

    #[test]
    fn matches_filter_response_accepts_valid_packet() {
        let proto = TP35ProProtocol;
        let mut data = vec![0u8; 34];
        data[OFFSET_CMD_TYPE] = READ;
        data[OFFSET_CMD] = CMD_PEQ_VALUES;
        data[OFFSET_NONCE] = 0x42;
        data[OFFSET_INDEX] = 3;
        assert!(proto.matches_filter_response(&data, 3, 0x42));
    }

    #[test]
    fn matches_filter_response_rejects_wrong_nonce() {
        let proto = TP35ProProtocol;
        let mut data = vec![0u8; 34];
        data[OFFSET_CMD_TYPE] = READ;
        data[OFFSET_CMD] = CMD_PEQ_VALUES;
        data[OFFSET_NONCE] = 0x42;
        data[OFFSET_INDEX] = 3;
        assert!(!proto.matches_filter_response(&data, 3, 0xFF));
    }

    #[test]
    fn matches_filter_response_rejects_short_packet() {
        let proto = TP35ProProtocol;
        assert!(!proto.matches_filter_response(&[READ, CMD_PEQ_VALUES], 0, 1));
    }

    #[test]
    fn matches_global_gain_response_accepts_valid_packet() {
        let proto = TP35ProProtocol;
        let mut data = vec![0u8; 6];
        data[OFFSET_CMD_TYPE] = READ;
        data[OFFSET_CMD] = CMD_GLOBAL_GAIN;
        data[OFFSET_GAIN_VALUE] = 3u8;
        assert!(proto.matches_global_gain_response(&data, 0));
        assert_eq!(proto.parse_global_gain_response(&data), Some(3i8));
    }

    #[test]
    fn parse_filter_response_too_short() {
        let proto = TP35ProProtocol;
        assert!(proto.parse_filter_response(&[0u8; 10]).is_none());
    }

    #[test]
    fn compute_iir_filter_produces_20_bytes() {
        assert_eq!(
            compute_iir_filter(FilterType::Peak, 1000.0, 5.0, 1.0).len(),
            20
        );
    }

    #[test]
    fn compute_iir_filter_lowpass_highpass_valid() {
        let lp_arr = compute_iir_filter(FilterType::LowPass, 1000.0, 0.0, 0.707);
        assert_eq!(lp_arr.len(), 20);
        let hp_arr = compute_iir_filter(FilterType::HighPass, 1000.0, 0.0, 0.707);
        assert_eq!(hp_arr.len(), 20);
    }

    #[test]
    fn parse_filter_packet_too_short() {
        assert!(parse_filter_packet(&[0u8; 10]).is_none());
    }
}
