use crate::hardware::dsp::{compute_iir_filter, convert_to_byte_array};
use crate::models::Filter;

pub const REPORT_ID: u8 = 0x4B;

pub const CMD_FLASH_EQ: u8 = 0x01;
pub const CMD_GLOBAL_GAIN: u8 = 0x03;
pub const CMD_PEQ_VALUES: u8 = 0x09;
pub const CMD_TEMP_WRITE: u8 = 0x0A;
pub const CMD_VERSION: u8 = 0x0C;

pub const READ: u8 = 0x80;
pub const WRITE: u8 = 0x01;
pub const END: u8 = 0x00;

pub const FILTER_SLOT: u8 = 101;

// Packet offsets for TP35Pro
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
pub const OFFSET_SLOT: usize = 35; // Slot byte in write packet

// Global Gain offsets
pub const OFFSET_GAIN_VALUE: usize = 4;

/// Trait defining hardware-specific packet layouts and constants.
pub trait DeviceProtocol: Send + Sync {
    fn report_id(&self) -> u8;
    fn cmd_version(&self) -> u8;
    fn cmd_peq_values(&self) -> u8;
    fn cmd_global_gain(&self) -> u8;
    fn cmd_temp_write(&self) -> u8;
    fn cmd_flash_eq(&self) -> u8;

    /// Build a request for a single filter index.
    fn build_filter_request(&self, index: u8, nonce: u8) -> Vec<u8>;

    /// Build a request for global gain.
    fn build_global_gain_request(&self, nonce: u8) -> Vec<u8>;

    /// Build a write packet for a filter.
    fn build_filter_write_packet(
        &self,
        index: u8,
        enabled: bool,
        freq: f64,
        gain: f64,
        q: f64,
        filter_type: u8,
    ) -> Vec<u8>;

    /// Build a write packet for global gain.
    fn build_global_gain_write_packet(&self, gain: i8) -> Vec<u8>;

    /// Build a temporary write commit packet.
    fn build_temp_write_packet(&self) -> Vec<u8>;

    /// Build a flash memory commit packet.
    fn build_flash_eq_packet(&self) -> Vec<u8>;

    /// Parse a filter packet from raw bytes (minus the report ID if present).
    fn parse_filter_packet(&self, data: &[u8]) -> Option<Filter>;
}

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

    fn build_filter_request(&self, index: u8, nonce: u8) -> Vec<u8> {
        vec![READ, CMD_PEQ_VALUES, nonce, 0x00, index, END]
    }

    fn build_global_gain_request(&self, _nonce: u8) -> Vec<u8> {
        // TP35 Pro ignores nonce for global gain
        vec![READ, CMD_GLOBAL_GAIN, 0x00, END]
    }

    fn build_filter_write_packet(
        &self,
        index: u8,
        enabled: bool,
        freq: f64,
        gain: f64,
        q: f64,
        filter_type: u8,
    ) -> Vec<u8> {
        let _ = enabled;

        let b_arr = compute_iir_filter(freq, gain, q);

        let mut packet = vec![WRITE, CMD_PEQ_VALUES, 0x18, 0x00, index, 0x00, 0x00];
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

    fn build_global_gain_write_packet(&self, gain: i8) -> Vec<u8> {
        vec![WRITE, CMD_GLOBAL_GAIN, 0x02, 0x00, gain as u8, END]
    }

    fn build_temp_write_packet(&self) -> Vec<u8> {
        vec![WRITE, CMD_TEMP_WRITE, 0x04, 0x00, 0x00, 0xFF, 0xFF, END]
    }

    fn build_flash_eq_packet(&self) -> Vec<u8> {
        vec![WRITE, CMD_FLASH_EQ, 0x01, FILTER_SLOT, END]
    }

    fn parse_filter_packet(&self, data: &[u8]) -> Option<Filter> {
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
