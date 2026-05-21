// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

//! Hardware protocol implementations for supported USB DAC devices.
//!
//! This module provides concrete implementations of the `DeviceProtocol` trait
//! defined in `core::device::protocol`. Each device has a unique protocol struct
//! that handles its specific binary packet formats and command bytes.

use crate::hardware::dsp::{compute_iir_filter, convert_to_2byte_array};

pub use crate::hardware::packet_format::{
    CMD_FLASH_EQ, CMD_GLOBAL_GAIN, CMD_PEQ_VALUES, CMD_TEMP_WRITE, CMD_VERSION, CONST_FLASH_EQ_LEN,
    CONST_GLOBAL_GAIN_LEN, CONST_PEQ_PAYLOAD_LEN, CONST_TEMP_WRITE_LEN, CONST_TEMP_WRITE_MAGIC_A,
    CONST_TEMP_WRITE_MAGIC_B, END, FILTER_SLOT, OFFSET_BIQUAD_START, OFFSET_CMD, OFFSET_CMD_TYPE,
    OFFSET_GAIN_VALUE, OFFSET_INDEX, OFFSET_NONCE, OFFSET_SLOT, READ, REPORT_ID, WRITE,
};

pub use crate::core::device::protocol::DeviceProtocol;

pub struct TP35ProProtocol;

impl DeviceProtocol for TP35ProProtocol {
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
    fn supports_per_band_enable(&self) -> bool {
        false
    }

    fn build_filter_read_request(&self, index: u8, nonce: u8) -> Vec<u8> {
        vec![READ, CMD_PEQ_VALUES, nonce, 0x00, index, END]
    }

    fn build_global_gain_request(&self, _nonce: u8) -> Vec<u8> {
        // TP35 Pro ignores nonce for global gain
        vec![READ, CMD_GLOBAL_GAIN, 0x00, END]
    }

    fn build_filter_write_packet(
        &self,
        index: u8,
        _enabled: bool,
        freq: f64,
        gain: f64,
        q: f64,
        filter_type: u8,
    ) -> Vec<u8> {
        let b_arr = compute_iir_filter(freq, gain, q);

        let mut packet = Vec::with_capacity(36);
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
        packet.extend_from_slice(&convert_to_2byte_array(freq.round() as i32));
        packet.extend_from_slice(&convert_to_2byte_array((q * 256.0).round() as i32));
        packet.extend_from_slice(&convert_to_2byte_array((gain * 256.0).round() as i32));
        packet.extend_from_slice(&[filter_type, 0x00, FILTER_SLOT, END]);

        packet
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

    fn build_temp_write_packet(&self) -> Vec<u8> {
        vec![
            WRITE,
            CMD_TEMP_WRITE,
            CONST_TEMP_WRITE_LEN,
            0x00,
            0x00,
            CONST_TEMP_WRITE_MAGIC_A,
            CONST_TEMP_WRITE_MAGIC_B,
            END,
        ]
    }

    fn build_flash_eq_packet(&self) -> Vec<u8> {
        vec![WRITE, CMD_FLASH_EQ, CONST_FLASH_EQ_LEN, FILTER_SLOT, END]
    }

    fn parse_filter_packet(&self, data: &[u8]) -> Option<crate::core::eq::Filter> {
        crate::hardware::dsp::parse_filter_packet(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tp35pro_build_filter_write_packet() {
        let proto = TP35ProProtocol;
        let packet = proto.build_filter_write_packet(0, true, 1000.0, 5.0, 1.0, 2);
        assert_eq!(packet[0], WRITE);
        assert_eq!(packet[1], CMD_PEQ_VALUES);
        assert_eq!(packet[4], 0); // index
        assert!(packet.len() > 30);
    }

    #[test]
    fn test_tp35pro_build_global_gain_write_packet() {
        let proto = TP35ProProtocol;
        let packet = proto.build_global_gain_write_packet(5);
        assert_eq!(packet[0], WRITE);
        assert_eq!(packet[1], CMD_GLOBAL_GAIN);
        assert_eq!(packet[4], 5);
    }

    #[test]
    fn test_tp35pro_build_temp_write_packet() {
        let proto = TP35ProProtocol;
        let packet = proto.build_temp_write_packet();
        assert_eq!(packet[1], CMD_TEMP_WRITE);
    }

    #[test]
    fn test_tp35pro_build_flash_eq_packet() {
        let proto = TP35ProProtocol;
        let packet = proto.build_flash_eq_packet();
        assert_eq!(packet[1], CMD_FLASH_EQ);
    }
}
