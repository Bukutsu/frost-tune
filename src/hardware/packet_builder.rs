use crate::error::Result;
use crate::hardware::hid::{delay_ms, send_report};
use crate::hardware::packet_format::{WriteTiming, END, READ};
use crate::hardware::protocol::DeviceProtocol;
use crate::models::Filter;
use hidapi::HidDevice;

pub fn init_device_session(device: &HidDevice, proto: &dyn DeviceProtocol) -> Result<()> {
    crate::hardware::hid::reset_nonce();
    send_report(
        device,
        &[READ, proto.cmd_version(), END][..],
        proto.report_id(),
    )?;
    delay_ms(50);
    let mut drain = [0u8; 64];
    while let Ok(count) = device.read_timeout(&mut drain[..], 20) {
        if count == 0 {
            break;
        }
    }
    Ok(())
}

pub fn write_filters_and_gain(
    device: &HidDevice,
    proto: &dyn DeviceProtocol,
    filters: &[Filter],
    global_gain: i8,
    timing: &WriteTiming,
) -> Result<()> {
    for (i, filter) in filters.iter().enumerate() {
        let packet = proto.build_filter_write_packet(
            i as u8,
            filter.enabled,
            filter.freq as f64,
            filter.gain,
            filter.q,
            filter.filter_type.into(),
        );
        send_report(device, &packet[..], proto.report_id())?;
        delay_ms(timing.per_filter_ms.max(timing.flood_delay_ms));
    }

    delay_ms(timing.batch_ms);

    let gain_packet = proto.build_global_gain_write_packet(global_gain);
    send_report(device, &gain_packet[..], proto.report_id())?;
    delay_ms(timing.global_gain_ms);

    Ok(())
}

pub fn commit_changes(
    device: &HidDevice,
    proto: &dyn DeviceProtocol,
    timing: &WriteTiming,
) -> Result<()> {
    let temp_packet = proto.build_temp_write_packet();
    send_report(device, &temp_packet[..], proto.report_id())?;
    delay_ms(timing.commit_ms);

    let flash_packet = proto.build_flash_eq_packet();
    send_report(device, &flash_packet[..], proto.report_id())?;

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
    }
}
