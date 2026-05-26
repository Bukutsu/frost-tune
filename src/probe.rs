// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

//! CLI probe command: read current PEQ data from a device and dump it to stdout.
//!
//! Usage: `frost-tune probe <vid> <pid> [--hex]`
//!
//! This is a developer diagnostic tool. It bypasses the async worker and talks
//! directly to the HID device. On Linux, the device must be accessible to the
//! current user (e.g. via udev rules or `pkexec frost-tune probe ...`).

use crate::core::FilterType;
use crate::hardware::hid::{pull_peq_internal, HidDeviceIo};
use crate::hardware::packet_builder::init_device_session;
use crate::hardware::registry;

pub struct ProbeOptions {
    pub vid: u16,
    pub pid: u16,
    pub hex: bool,
}

#[derive(Debug)]
struct RawHidDevice(hidapi::HidDevice);

impl HidDeviceIo for RawHidDevice {
    fn write(&self, data: &[u8]) -> std::result::Result<usize, hidapi::HidError> {
        hidapi::HidDevice::write(&self.0, data)
    }
    fn read_timeout(
        &self,
        data: &mut [u8],
        timeout_ms: i32,
    ) -> std::result::Result<usize, hidapi::HidError> {
        hidapi::HidDevice::read_timeout(&self.0, data, timeout_ms)
    }
}

pub fn run(opts: ProbeOptions) -> Result<(), String> {
    let profile = registry::get_profile(opts.vid, opts.pid).ok_or_else(|| {
        let known: Vec<String> = registry::list_profiles()
            .iter()
            .map(|p| {
                format!(
                    "{:04x}:{:04x} ({})",
                    p.vendor_id(),
                    p.product_id(),
                    p.name()
                )
            })
            .collect();
        format!(
            "Device {:04x}:{:04x} is not in the supported registry. Known devices:\n  {}",
            opts.vid,
            opts.pid,
            if known.is_empty() {
                "(none)".to_string()
            } else {
                known.join("\n  ")
            }
        )
    })?;

    let api = hidapi::HidApi::new().map_err(|e| format!("Failed to initialize HID API: {}", e))?;

    let device = api.open(opts.vid, opts.pid).map_err(|e| {
        format!(
            "Failed to open device {:04x}:{:04x}: {}",
            opts.vid, opts.pid, e
        )
    })?;
    device
        .set_blocking_mode(true)
        .map_err(|e| format!("Failed to set blocking mode: {}", e))?;

    let proto = profile.protocol();
    let caps = profile.capabilities();
    let wrapper = RawHidDevice(device);

    eprintln!(
        "Connected to {} ({:04x}:{:04x})",
        profile.name(),
        opts.vid,
        opts.pid
    );
    eprintln!(
        "  Bands: {}, Global gain range: {}..{} dB",
        caps.num_bands, caps.global_gain_range.0, caps.global_gain_range.1
    );
    eprintln!(
        "  Band gain range: {:.0}..{:.0} dB, Freq range: {}..{} Hz, Q range: {:.1}..{:.1}",
        caps.band_gain_range.0,
        caps.band_gain_range.1,
        caps.freq_range.0,
        caps.freq_range.1,
        caps.q_range.0,
        caps.q_range.1
    );

    let init_packets = proto.build_init_packets();
    if opts.hex && !init_packets.is_empty() {
        eprintln!("\n--- Init packets ---");
        for (i, pkt) in init_packets.iter().enumerate() {
            eprintln!("  [{}] {:02x?}", i, pkt);
        }
    }

    init_device_session(&wrapper, proto.as_ref())
        .map_err(|e| format!("Init session failed: {}", e.message))?;

    if opts.hex {
        eprintln!("\n--- Filter read requests ---");
    }

    eprintln!("\nPulling PEQ data...");
    let dummy_check = || false;
    let peq = pull_peq_internal(
        &wrapper,
        proto.as_ref(),
        false,
        caps.num_bands,
        &dummy_check,
    )
    .map_err(|e| format!("Failed to read PEQ state: {}", e.message))?;

    println!("\n=== {} ===", profile.name());
    println!("Global Gain: {} dB", peq.global_gain);
    println!(
        "{:<6} {:<8} {:<10} {:<8} {:<8} {:<10}",
        "Band", "Enabled", "Freq (Hz)", "Gain (dB)", "Q", "Type"
    );
    println!("{}", "-".repeat(60));

    for filter in &peq.filters {
        let enabled = if filter.enabled { "Yes" } else { "No" };
        let ftype = filter_type_display(filter.filter_type);
        println!(
            "{:<6} {:<8} {:<10} {:<8.1} {:<8.2} {:<10}",
            filter.index, enabled, filter.freq, filter.gain, filter.q, ftype
        );
    }

    if opts.hex {
        println!("\n--- Raw band data ---");
        for filter in &peq.filters {
            println!(
                "  Band {}: enabled={} freq={} gain={:.1} q={:.2} type={:?}",
                filter.index,
                filter.enabled,
                filter.freq,
                filter.gain,
                filter.q,
                filter.filter_type
            );
        }
    }

    Ok(())
}

fn filter_type_display(ft: FilterType) -> &'static str {
    match ft {
        FilterType::Peak => "Peak",
        FilterType::LowShelf => "Low Shelf",
        FilterType::HighShelf => "High Shelf",
        FilterType::HighPass => "High Pass",
        FilterType::LowPass => "Low Pass",
    }
}
