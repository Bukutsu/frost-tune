// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

//! Device protocol trait defining the interface for hardware-specific packet handling.
//!
//! Each supported USB DAC device implements this trait to handle its specific binary protocol,
//! command bytes, and packet formats. This allows new devices to be added without modifying
//! the core communication logic.

use crate::core::eq::Filter;
use crate::hardware::packet_format::{ReadTiming, WriteTiming};

/// Trait defining hardware-specific packet layouts and constants.
///
/// **Contributor Guide: Implementing a New Protocol**
/// To support a new USB DAC, you must create a new struct (e.g., `MyNewDeviceProtocol`)
/// and implement this trait for it.
///
/// Hardware devices often have unique binary payloads and command bytes for reading
/// and writing EQ filters and global gains. Use USB packet capture tools (like Wireshark or USBPcap)
/// on the official device software to reverse-engineer these payloads.
///
/// Note: Ensure your protocol implementation handles the required endianness and
/// offset indexing accurately as expected by the target hardware.
pub trait DeviceProtocol: Send + Sync {
    fn report_id(&self) -> u8;
    fn cmd_version(&self) -> u8;
    fn cmd_peq_values(&self) -> u8;
    fn cmd_global_gain(&self) -> u8;
    fn cmd_temp_write(&self) -> u8;
    fn cmd_flash_eq(&self) -> u8;

    /// Dynamic limits exposed by the device protocol
    fn num_bands(&self) -> usize {
        10
    }
    fn freq_range(&self) -> (u16, u16) {
        (20, 20000)
    }
    fn gain_range(&self) -> (f64, f64) {
        (-10.0, 10.0)
    }
    fn q_range(&self) -> (f64, f64) {
        (0.1, 10.0)
    }

    /// Whether the device supports enabling/disabling individual filter bands.
    fn supports_per_band_enable(&self) -> bool {
        true
    }

    /// Default timings for reading from this device.
    fn read_timing(&self) -> ReadTiming {
        ReadTiming::default()
    }

    /// Default timings for writing to this device.
    fn write_timing(&self) -> WriteTiming {
        WriteTiming::default()
    }

    /// Build a request payload to read a single filter at the specified index.
    /// The `nonce` is typically used to correlate the request with the response.
    fn build_filter_read_request(&self, index: u8, nonce: u8) -> Vec<u8>;

    /// Build a request payload to read the global gain value from the device.
    fn build_global_gain_request(&self, nonce: u8) -> Vec<u8>;

    /// Build a packet to write a single filter to the device's volatile memory.
    /// * `index` - The 0-based index of the filter band.
    /// * `enabled` - Whether the filter band is active.
    /// * `freq` - The center frequency of the filter.
    /// * `gain` - The gain value of the filter in dB.
    /// * `q` - The Q-factor (bandwidth) of the filter.
    /// * `filter_type` - The specific filter type byte (e.g., Peaking, Low Shelf, High Shelf).
    fn build_filter_write_packet(
        &self,
        index: u8,
        enabled: bool,
        freq: f64,
        gain: f64,
        q: f64,
        filter_type: u8,
    ) -> Vec<u8>;

    /// Build a packet to write the global gain value to the device's volatile memory.
    fn build_global_gain_write_packet(&self, gain: i8) -> Vec<u8>;

    /// Build a packet that commits the current volatile EQ configuration
    /// into a temporary active state on the hardware.
    fn build_temp_write_packet(&self) -> Vec<u8>;

    /// Build a packet that commits the current EQ configuration to the
    /// hardware's persistent flash memory so it survives a power cycle.
    fn build_flash_eq_packet(&self) -> Vec<u8>;

    /// Parse a raw byte payload (excluding the report ID, if present) returned
    /// by the device in response to a filter read request, extracting it into a `Filter` struct.
    /// Return `None` if the packet is invalid or unparseable.
    fn parse_filter_packet(&self, data: &[u8]) -> Option<Filter>;
}
