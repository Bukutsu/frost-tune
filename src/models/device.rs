use serde::{Deserialize, Serialize};

use crate::hardware::protocol::{DeviceProtocol, TP35ProProtocol};

/// A declarative macro that centralizes the registration of all supported USB DAC devices.
///
/// This macro generates the `Device` enum and automatically implements all necessary
/// matching logic for vendor IDs, product IDs, human-readable names, and protocol instantiation.
/// By using this macro, contributors only need to add a single block to support a new device.
macro_rules! define_devices {
    (
        $(
            $variant:ident {
                name: $name:expr,
                vid: $vid:expr,
                pid: $pid:expr,
                protocol: $proto:ident,
            }
        ),* $(,)?
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Device {
            $($variant,)*
            Unknown,
        }

        impl Device {
            /// Returns a boxed instance of the hardware protocol used to communicate with this device.
            pub fn protocol(&self) -> Option<Box<dyn DeviceProtocol>> {
                match self {
                    $(Device::$variant => Some(Box::new($proto)),)*
                    Device::Unknown => None,
                }
            }

            /// Identifies the device based on its USB Vendor ID and Product ID.
            pub fn from_vid_pid(vid: u16, pid: u16) -> Self {
                match (vid, pid) {
                    $( ($vid, $pid) => Device::$variant, )*
                    _ => Device::Unknown,
                }
            }

            /// Returns the USB Vendor ID associated with this device.
            pub fn vendor_id(&self) -> u16 {
                match self {
                    $(Device::$variant => $vid,)*
                    Device::Unknown => 0,
                }
            }

            /// Returns the USB Product ID associated with this device.
            pub fn product_id(&self) -> u16 {
                match self {
                    $(Device::$variant => $pid,)*
                    Device::Unknown => 0,
                }
            }

            /// Returns the human-readable display name of the device.
            pub fn name(&self) -> &'static str {
                match self {
                    $(Device::$variant => $name,)*
                    Device::Unknown => "Unknown Device",
                }
            }
        }
    };
}

define_devices! {
    TP35Pro {
        name: "EPZ TP35 Pro",
        vid: 0x3302,
        pid: 0x43E6,
        protocol: TP35ProProtocol,
    },

    // =========================================================================
    // CONTRIBUTOR GUIDE: ADDING A NEW DEVICE
    // =========================================================================
    // To add support for a new device, simply define it here and ensure you
    // have implemented a protocol for it in `src/hardware/protocol.rs`.
    //
    // DO NOT DELETE THIS EXAMPLE BLOCK. You can copy it for your PR.
    //
    // ExampleDeviceVariant {
    //     name: "Manufacturer Device Name",
    //     vid: 0x1234, // The USB Vendor ID (hex)
    //     pid: 0x5678, // The USB Product ID (hex)
    //     protocol: ExampleDeviceProtocol, // The struct implementing `DeviceProtocol`
    // },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub path: String,
    pub manufacturer: Option<String>,
}
