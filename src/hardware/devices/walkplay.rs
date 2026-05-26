// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

//! Generic Walkplay DSP protocol family: profiles for Moondrop Dawn Pro and Truthear KEYX.
//!
//! Reuses the core DSP math and DeviceProtocol implementation from `tp35pro.rs`
//! since they share the exact same wire format and chipset.

use crate::core::device::capabilities::{DeviceCapabilities, FilterTypeFlags};
use crate::core::device::profile::DeviceProfile;
use crate::core::device::protocol::DeviceProtocol;

/// Shared protocol implementation for all Walkplay family devices.
pub struct WalkplayProtocol;

impl DeviceProtocol for WalkplayProtocol {
    fn report_id(&self) -> u8 {
        crate::hardware::devices::tp35pro::REPORT_ID
    }

    fn write_timing(&self) -> crate::core::device::timing::WriteTiming {
        crate::core::device::timing::WriteTiming {
            commit_step_ms: 500,
            ..crate::core::device::timing::WriteTiming::default()
        }
    }

    fn is_default_state(&self, peq: &crate::core::eq::PEQData) -> bool {
        let all_disabled = peq.filters.iter().all(|f| !f.enabled);
        let has_default_gain = peq.global_gain == 0;
        let all_default_freq = peq.filters.iter().all(|f| f.freq == 100);
        all_disabled && has_default_gain && all_default_freq
    }

    fn build_init_packets(&self) -> Vec<Vec<u8>> {
        vec![vec![
            crate::hardware::devices::tp35pro::READ,
            crate::hardware::devices::tp35pro::CMD_VERSION,
            crate::hardware::devices::tp35pro::END,
        ]]
    }

    fn build_filter_read_request(&self, index: u8, nonce: u8) -> Vec<u8> {
        vec![
            crate::hardware::devices::tp35pro::READ,
            crate::hardware::devices::tp35pro::CMD_PEQ_VALUES,
            nonce,
            0x00,
            index,
            crate::hardware::devices::tp35pro::END,
        ]
    }

    fn matches_filter_response(&self, data: &[u8], index: u8, nonce: u8) -> bool {
        data.len() >= 34
            && data[crate::hardware::devices::tp35pro::OFFSET_CMD_TYPE]
                == crate::hardware::devices::tp35pro::READ
            && data[crate::hardware::devices::tp35pro::OFFSET_CMD]
                == crate::hardware::devices::tp35pro::CMD_PEQ_VALUES
            && data[crate::hardware::devices::tp35pro::OFFSET_NONCE] == nonce
            && data[crate::hardware::devices::tp35pro::OFFSET_INDEX] == index
    }

    fn parse_filter_response(&self, data: &[u8]) -> Option<crate::core::eq::Filter> {
        crate::hardware::devices::tp35pro::parse_filter_packet(data)
    }

    fn build_filter_write_packet(
        &self,
        index: u8,
        filter: &crate::core::eq::Filter,
        dsp_sample_rate: f64,
    ) -> Vec<u8> {
        let b_arr = crate::hardware::devices::tp35pro::compute_iir_filter(
            filter.filter_type,
            filter.freq as f64,
            filter.gain,
            filter.q,
            dsp_sample_rate,
        );
        let filter_type_byte: u8 = filter.filter_type.into();

        let mut packet = Vec::with_capacity(37);
        packet.extend_from_slice(&[
            crate::hardware::devices::tp35pro::WRITE,
            crate::hardware::devices::tp35pro::CMD_PEQ_VALUES,
            crate::hardware::devices::tp35pro::CONST_PEQ_PAYLOAD_LEN,
            0x00,
            index,
            0x00,
            0x00,
        ]);
        packet.extend_from_slice(&b_arr);
        packet.extend_from_slice(&crate::hardware::devices::tp35pro::convert_to_2byte_array(
            filter.freq as i32,
        ));
        packet.extend_from_slice(&crate::hardware::devices::tp35pro::convert_to_2byte_array(
            (filter.q * 256.0).round() as i32,
        ));
        packet.extend_from_slice(&crate::hardware::devices::tp35pro::convert_to_2byte_array(
            (filter.gain * 256.0).round() as i32,
        ));
        packet.extend_from_slice(&[
            filter_type_byte,
            0x00,
            crate::hardware::devices::tp35pro::FILTER_SLOT,
            crate::hardware::devices::tp35pro::END,
        ]);

        packet
    }

    fn build_global_gain_request(&self, _nonce: u8) -> Vec<u8> {
        vec![
            crate::hardware::devices::tp35pro::READ,
            crate::hardware::devices::tp35pro::CMD_GLOBAL_GAIN,
            0x00,
            crate::hardware::devices::tp35pro::END,
        ]
    }

    fn matches_global_gain_response(&self, data: &[u8], _nonce: u8) -> bool {
        data.len() >= 6
            && data[crate::hardware::devices::tp35pro::OFFSET_CMD_TYPE]
                == crate::hardware::devices::tp35pro::READ
            && data[crate::hardware::devices::tp35pro::OFFSET_CMD]
                == crate::hardware::devices::tp35pro::CMD_GLOBAL_GAIN
    }

    fn parse_global_gain_response(&self, data: &[u8]) -> Option<i8> {
        if data.len() > crate::hardware::devices::tp35pro::OFFSET_GAIN_VALUE {
            Some(data[crate::hardware::devices::tp35pro::OFFSET_GAIN_VALUE] as i8)
        } else {
            None
        }
    }

    fn build_global_gain_write_packet(&self, gain: i8) -> Vec<u8> {
        vec![
            crate::hardware::devices::tp35pro::WRITE,
            crate::hardware::devices::tp35pro::CMD_GLOBAL_GAIN,
            crate::hardware::devices::tp35pro::CONST_GLOBAL_GAIN_LEN,
            0x00,
            gain as u8,
            crate::hardware::devices::tp35pro::END,
        ]
    }

    fn build_commit_packets(&self) -> Vec<Vec<u8>> {
        vec![
            vec![
                crate::hardware::devices::tp35pro::WRITE,
                crate::hardware::devices::tp35pro::CMD_TEMP_WRITE,
                crate::hardware::devices::tp35pro::CONST_TEMP_WRITE_LEN,
                0x00,
                0x00,
                crate::hardware::devices::tp35pro::CONST_TEMP_WRITE_MAGIC_A,
                crate::hardware::devices::tp35pro::CONST_TEMP_WRITE_MAGIC_B,
                crate::hardware::devices::tp35pro::END,
            ],
            vec![
                crate::hardware::devices::tp35pro::WRITE,
                crate::hardware::devices::tp35pro::CMD_FLASH_EQ,
                crate::hardware::devices::tp35pro::CONST_FLASH_EQ_LEN,
                crate::hardware::devices::tp35pro::FILTER_SLOT,
                crate::hardware::devices::tp35pro::END,
            ],
        ]
    }
}

pub struct DawnProProfile;

impl DeviceProfile for DawnProProfile {
    fn name(&self) -> &'static str {
        "Moondrop Dawn Pro"
    }

    fn vendor_id(&self) -> u16 {
        0x2FC6
    }

    fn product_id(&self) -> u16 {
        0xDF30
    }

    fn capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities {
            num_bands: 8,
            global_gain_range: (-20, 0),
            band_gain_range: (-12.0, 12.0),
            freq_range: (20, 20000),
            q_range: (0.1, 10.0),
            supported_filter_types: FilterTypeFlags::PEAK
                | FilterTypeFlags::LOW_SHELF
                | FilterTypeFlags::HIGH_SHELF,
            supports_per_band_enable: false,
            dsp_sample_rate: 96000.0,
            gain_tolerance: 0.1,
            freq_tolerance: 1,
            q_tolerance: 0.05,
        }
    }

    fn protocol(&self) -> Box<dyn DeviceProtocol> {
        Box::new(WalkplayProtocol)
    }
}

pub struct TruthearKeyxProfile;

impl DeviceProfile for TruthearKeyxProfile {
    fn name(&self) -> &'static str {
        "Truthear KEYX"
    }

    fn vendor_id(&self) -> u16 {
        0x0D8C
    }

    fn product_id(&self) -> u16 {
        0x0210
    }

    fn capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities {
            num_bands: 8,
            global_gain_range: (-20, 0),
            band_gain_range: (-12.0, 12.0),
            freq_range: (20, 20000),
            q_range: (0.1, 10.0),
            supported_filter_types: FilterTypeFlags::PEAK
                | FilterTypeFlags::LOW_SHELF
                | FilterTypeFlags::HIGH_SHELF,
            supports_per_band_enable: false,
            dsp_sample_rate: 96000.0,
            gain_tolerance: 0.1,
            freq_tolerance: 1,
            q_tolerance: 0.05,
        }
    }

    fn protocol(&self) -> Box<dyn DeviceProtocol> {
        Box::new(WalkplayProtocol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::eq::{Filter, FilterType};

    #[test]
    fn test_dawn_pro_capabilities() {
        let profile = DawnProProfile;
        let caps = profile.capabilities();
        assert_eq!(caps.num_bands, 8);
        assert_eq!(profile.vendor_id(), 0x2FC6);
        assert_eq!(profile.product_id(), 0xDF30);
    }

    #[test]
    fn test_keyx_capabilities() {
        let profile = TruthearKeyxProfile;
        let caps = profile.capabilities();
        assert_eq!(caps.num_bands, 8);
        assert_eq!(profile.vendor_id(), 0x0D8C);
        assert_eq!(profile.product_id(), 0x0210);
    }

    #[test]
    fn test_walkplay_packet_equivalence() {
        let ref_proto = crate::hardware::devices::tp35pro::TP35ProProtocol;
        let new_proto = WalkplayProtocol;

        let filter = Filter {
            index: 0,
            enabled: true,
            filter_type: FilterType::Peak,
            freq: 1000,
            gain: 5.0,
            q: 1.0,
        };

        let ref_packet = ref_proto.build_filter_write_packet(0, &filter, 96000.0);
        let new_packet = new_proto.build_filter_write_packet(0, &filter, 96000.0);

        assert_eq!(ref_packet, new_packet);
    }
}
