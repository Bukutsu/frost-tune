// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

use crate::core::device::capabilities::{DeviceCapabilities, FilterTypeFlags};
use crate::core::device::protocol::DeviceProtocol;

/// A declarative macro that centralizes the registration of all supported USB DAC devices.
///
/// This macro generates the `Device` enum and automatically implements all matching
/// logic for vendor IDs, product IDs, names, capabilities, and protocol instantiation.
/// To add a new device, add a single block here and implement `DeviceProtocol` for its
/// protocol struct. See `CONTRIBUTING_DEVICES.md` for the full walkthrough.
macro_rules! define_devices {
    (
        $(
            $variant:ident {
                name: $name:expr,
                vid: $vid:expr,
                pid: $pid:expr,
                protocol: $proto:ident,
                supported_filter_types: $filter_types:expr,
                supports_per_band_enable: $supports_per_band_enable:expr,
                min_global_gain: $min_global_gain:expr,
                max_global_gain: $max_global_gain:expr,
                num_bands: $num_bands:expr,
                min_band_gain: $min_band_gain:expr,
                max_band_gain: $max_band_gain:expr,
                min_freq: $min_freq:expr,
                max_freq: $max_freq:expr,
                min_q: $min_q:expr,
                max_q: $max_q:expr,
            }
        ),* $(,)?
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Device {
            $($variant,)*
            Unknown,
        }

        impl Device {
            /// Returns a boxed instance of the hardware protocol for this device.
            pub fn protocol(&self) -> Option<Box<dyn DeviceProtocol>> {
                match self {
                    $(Device::$variant => Some(Box::new($proto)),)*
                    Device::Unknown => None,
                }
            }

            /// Identifies the device from its USB Vendor ID and Product ID.
            pub fn from_vid_pid(vid: u16, pid: u16) -> Self {
                match (vid, pid) {
                    $( ($vid, $pid) => Device::$variant, )*
                    _ => Device::Unknown,
                }
            }

            pub fn vendor_id(&self) -> u16 {
                match self {
                    $(Device::$variant => $vid,)*
                    Device::Unknown => 0,
                }
            }

            pub fn product_id(&self) -> u16 {
                match self {
                    $(Device::$variant => $pid,)*
                    Device::Unknown => 0,
                }
            }

            pub fn name(&self) -> &'static str {
                match self {
                    $(Device::$variant => $name,)*
                    Device::Unknown => "Unknown Device",
                }
            }

            /// Full capability profile. Use individual accessors for single-field lookups.
            pub fn capabilities(&self) -> DeviceCapabilities {
                match self {
                    $(Device::$variant => DeviceCapabilities {
                        num_bands: $num_bands,
                        global_gain_range: ($min_global_gain, $max_global_gain),
                        band_gain_range: ($min_band_gain, $max_band_gain),
                        freq_range: ($min_freq, $max_freq),
                        q_range: ($min_q, $max_q),
                        supported_filter_types: $filter_types,
                        supports_per_band_enable: $supports_per_band_enable,
                    },)*
                    Device::Unknown => DeviceCapabilities {
                        num_bands: crate::models::NUM_BANDS,
                        global_gain_range: (crate::models::MIN_GLOBAL_GAIN, crate::models::MAX_GLOBAL_GAIN),
                        band_gain_range: (crate::models::MIN_BAND_GAIN, crate::models::MAX_BAND_GAIN),
                        freq_range: (crate::models::MIN_FREQ, crate::models::MAX_FREQ),
                        q_range: (crate::models::MIN_Q, crate::models::MAX_Q),
                        supported_filter_types: FilterTypeFlags::PEAK
                            | FilterTypeFlags::LOW_SHELF
                            | FilterTypeFlags::HIGH_SHELF
                            | FilterTypeFlags::LOW_PASS
                            | FilterTypeFlags::HIGH_PASS,
                        supports_per_band_enable: true,
                    },
                }
            }

            pub fn supported_filter_types(&self) -> FilterTypeFlags {
                self.capabilities().supported_filter_types
            }

            pub fn supports_per_band_enable(&self) -> bool {
                match self {
                    $(Device::$variant => $supports_per_band_enable,)*
                    Device::Unknown => true,
                }
            }

            pub fn min_global_gain(&self) -> i8 {
                match self {
                    $(Device::$variant => $min_global_gain,)*
                    Device::Unknown => crate::models::MIN_GLOBAL_GAIN,
                }
            }

            pub fn max_global_gain(&self) -> i8 {
                match self {
                    $(Device::$variant => $max_global_gain,)*
                    Device::Unknown => crate::models::MAX_GLOBAL_GAIN,
                }
            }

            pub fn num_bands(&self) -> usize {
                match self {
                    $(Device::$variant => $num_bands,)*
                    Device::Unknown => crate::models::NUM_BANDS,
                }
            }

            pub fn band_gain_range(&self) -> (f64, f64) {
                match self {
                    $(Device::$variant => ($min_band_gain, $max_band_gain),)*
                    Device::Unknown => (crate::models::MIN_BAND_GAIN, crate::models::MAX_BAND_GAIN),
                }
            }

            pub fn freq_range(&self) -> (u16, u16) {
                match self {
                    $(Device::$variant => ($min_freq, $max_freq),)*
                    Device::Unknown => (crate::models::MIN_FREQ, crate::models::MAX_FREQ),
                }
            }

            pub fn q_range(&self) -> (f64, f64) {
                match self {
                    $(Device::$variant => ($min_q, $max_q),)*
                    Device::Unknown => (crate::models::MIN_Q, crate::models::MAX_Q),
                }
            }

            pub fn global_gain_range(&self) -> (i8, i8) {
                (self.min_global_gain(), self.max_global_gain())
            }
        }
    };
}

use crate::core::device::tp35pro::TP35ProProtocol;

define_devices! {
    TP35Pro {
        name: "EPZ TP35 Pro",
        vid: 0x3302,
        pid: 0x43E6,
        protocol: TP35ProProtocol,
        supported_filter_types: FilterTypeFlags::PEAK
            | FilterTypeFlags::LOW_SHELF
            | FilterTypeFlags::HIGH_SHELF,
        supports_per_band_enable: false,
        min_global_gain: -16,
        max_global_gain: 6,
        num_bands: 10,
        min_band_gain: -10.0,
        max_band_gain: 10.0,
        min_freq: 20,
        max_freq: 20000,
        min_q: 0.1,
        max_q: 10.0,
    },

    // =========================================================================
    // CONTRIBUTOR GUIDE: ADDING A NEW DEVICE
    // =========================================================================
    // 1. Create src/core/device/<vendor>/mod.rs and implement DeviceProtocol.
    // 2. Add your struct name to the `use` statement above this block.
    // 3. Copy-paste and fill in a block below — one block per VID/PID.
    // 4. Add a udev rule in packaging/udev/99-frost-tune.rules.
    // See CONTRIBUTING_DEVICES.md for the full walkthrough.
    //
    // ExampleDevice {
    //     name: "Manufacturer Model Name",
    //     vid: 0x1234,
    //     pid: 0x5678,
    //     protocol: ExampleDeviceProtocol,
    //     supported_filter_types: FilterTypeFlags::PEAK,
    //     supports_per_band_enable: false,
    //     min_global_gain: -10,
    //     max_global_gain: 6,
    //     num_bands: 10,
    //     min_band_gain: -10.0,
    //     max_band_gain: 10.0,
    //     min_freq: 20,
    //     max_freq: 20000,
    //     min_q: 0.1,
    //     max_q: 10.0,
    // },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub path: String,
    pub manufacturer: Option<String>,
}
