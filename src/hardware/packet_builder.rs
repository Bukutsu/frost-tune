// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::core::device::protocol::DeviceProtocol;
use crate::core::device::timing::WriteTiming;
use crate::core::Filter;
use crate::error::{AppError, ErrorKind, Result};
use crate::hardware::hid::{delay_ms, send_report, DeviceSession, HidDeviceIo};

/// Sends the device's init sequence (version ping / wake), drains stale USB frames,
/// and returns a fresh `DeviceSession` with its nonce counter reset to 1.
/// Every read and write operation must start here.
pub fn init_device_session(
    device: &dyn HidDeviceIo,
    proto: &dyn DeviceProtocol,
) -> Result<DeviceSession> {
    for packet in proto.build_init_packets() {
        send_report(device, &packet, proto.report_id())?;
    }
    delay_ms(50);
    let mut drain = [0u8; 64];
    let mut iterations = 0;
    while let Ok(count) = device.read_timeout(&mut drain[..], 20) {
        if count == 0 || iterations > 100 {
            break;
        }
        iterations += 1;
    }
    Ok(DeviceSession::new())
}

pub fn write_filters_and_gain(
    device: &dyn HidDeviceIo,
    proto: &dyn DeviceProtocol,
    filters: &[Filter],
    global_gain: i8,
    timing: &WriteTiming,
) -> Result<()> {
    if filters.len() > proto.num_bands() {
        return Err(AppError::new(
            ErrorKind::InvalidPayload,
            format!(
                "Payload exceeds device capacity. Provided: {}, max allowed: {}",
                filters.len(),
                proto.num_bands()
            ),
        ));
    }

    for (i, filter) in filters.iter().enumerate() {
        let packet = proto.build_filter_write_packet(i as u8, filter);
        send_report(device, &packet, proto.report_id())?;
        delay_ms(timing.per_filter_ms.max(timing.flood_delay_ms));
    }

    delay_ms(timing.batch_ms);

    let gain_packet = proto.build_global_gain_write_packet(global_gain);
    send_report(device, &gain_packet, proto.report_id())?;
    delay_ms(timing.global_gain_ms);

    Ok(())
}

/// Sends the full commit sequence returned by `proto.build_commit_packets()`,
/// applying `timing.commit_step_ms` delay after each packet.
pub fn commit_changes(
    device: &dyn HidDeviceIo,
    proto: &dyn DeviceProtocol,
    timing: &WriteTiming,
) -> Result<()> {
    for packet in proto.build_commit_packets() {
        send_report(device, &packet, proto.report_id())?;
        delay_ms(timing.commit_step_ms);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_timing_default() {
        let timing = WriteTiming::default();
        assert!(timing.per_filter_ms > 0);
        assert!(timing.batch_ms > 0);
        assert!(timing.commit_step_ms > 0);
    }
}
