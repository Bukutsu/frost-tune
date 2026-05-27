// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::core::device::protocol::DeviceProtocol;
use crate::core::{DeviceCapabilities, Filter, PEQData};
use crate::error::{AppError, ErrorKind, Result};
use crate::hardware::device_io::PhysicalInterface;
use crate::hardware::hid::{delay_ms, send_report};
use crate::hardware::operations::{
    compare_peq, compare_peq_exclude_gain, pull_peq_data, rollback_and_verify,
};
use crate::hardware::packet_builder::{
    commit_changes, init_device_session, write_filters_and_gain,
};
use crate::hardware::DeviceProfile;
use crate::hardware::PushPayload;

pub fn pull_with_retry(
    device: &dyn PhysicalInterface,
    proto: &dyn DeviceProtocol,
    strict: bool,
    num_bands: usize,
    check_in: &dyn Fn() -> bool,
) -> Result<PEQData> {
    if check_in() {
        return Err(AppError::new(ErrorKind::OperationCancelled, "Cancelled"));
    }

    let wake_request = proto.build_global_gain_request(WAKE_NONCE);
    let framer_box = proto.framer();
    let framer = framer_box.as_ref();
    if let Err(e) = crate::hardware::hid::send_report(device, &wake_request[..], framer) {
        log::warn!("pull wake request failed: {}", e.message);
    }
    delay_ms(proto.read_timing().wake_delay_ms);
    let first_result = pull_peq_data(device, proto, strict, num_bands, check_in);

    let needs_retry = match &first_result {
        Ok(peq) => proto.is_default_state(peq),
        Err(_) => true,
    };

    if needs_retry {
        if check_in() {
            return Err(AppError::new(ErrorKind::OperationCancelled, "Cancelled"));
        }
        delay_ms(proto.read_timing().pull_retry_delay_ms);
        match pull_peq_data(device, proto, strict, num_bands, check_in) {
            Ok(peq) => Ok(peq),
            Err(e) => {
                if first_result.is_ok() {
                    first_result
                } else {
                    Err(e)
                }
            }
        }
    } else {
        first_result
    }
}

const WAKE_NONCE: u8 = 0x01;
const VERIFY_RETRY_COUNT: usize = 3;

fn wake_device(device: &dyn PhysicalInterface, proto: &dyn DeviceProtocol) {
    let wake = proto.build_global_gain_request(WAKE_NONCE);
    let framer_box = proto.framer();
    let framer = framer_box.as_ref();
    if let Err(e) = send_report(device, &wake[..], framer) {
        log::warn!("wake request failed: {}", e.message);
    }
    delay_ms(proto.read_timing().wake_delay_ms);
}

fn do_write_and_commit(
    device: &dyn PhysicalInterface,
    proto: &dyn DeviceProtocol,
    payload: &PushPayload,
    num_bands: usize,
    dsp_sample_rate: f64,
) -> Result<()> {
    let timing = proto.write_timing();
    init_device_session(device, proto)?;
    write_filters_and_gain(
        device,
        proto,
        &payload.filters,
        payload.global_gain.unwrap_or(0),
        &timing,
        num_bands,
        dsp_sample_rate,
    )?;
    commit_changes(device, proto, &timing)
}

#[allow(clippy::too_many_arguments)]
fn verify_or_rollback(
    device: &dyn PhysicalInterface,
    proto: &dyn DeviceProtocol,
    expected_filters: &[Filter],
    expected_gain: Option<i8>,
    snapshot: &PEQData,
    num_bands: usize,
    dsp_sample_rate: f64,
    caps: &DeviceCapabilities,
    check_in: &dyn Fn() -> bool,
) -> Result<PEQData> {
    let backoff_base = proto.read_timing().verify_backoff_base_ms;
    for attempt in 0..VERIFY_RETRY_COUNT {
        let backoff_ms = backoff_base.saturating_mul(2u64.saturating_pow(attempt as u32));
        delay_ms(backoff_ms);
        match pull_peq_data(device, proto, true, num_bands, check_in) {
            Ok(read_back) => {
                let cmp = if let Some(gain) = expected_gain {
                    compare_peq(&read_back, expected_filters, gain, caps)
                } else {
                    compare_peq_exclude_gain(&read_back, expected_filters, caps)
                };
                if cmp.is_ok() {
                    return Ok(read_back);
                }
            }
            Err(e) => {
                rollback_and_verify(
                    device,
                    proto,
                    snapshot,
                    num_bands,
                    dsp_sample_rate,
                    caps,
                    check_in,
                )
                .map_err(|r| {
                    AppError::new(
                        ErrorKind::RollbackFailed,
                        format!(
                            "Verify read error: {} | rollback failed: {}",
                            e.message, r.message
                        ),
                    )
                })?;
                return Err(e);
            }
        }
    }
    rollback_and_verify(
        device,
        proto,
        snapshot,
        num_bands,
        dsp_sample_rate,
        caps,
        check_in,
    )
    .map_err(|r| {
        AppError::new(
            ErrorKind::RollbackFailed,
            format!(
                "Verification failed: settings did not match | rollback failed: {}",
                r.message
            ),
        )
    })?;
    Err(AppError::new(
        ErrorKind::VerifyFailed,
        "Verification failed: settings did not match",
    ))
}

pub fn push_with_verify(
    device: &dyn PhysicalInterface,
    profile: &dyn DeviceProfile,
    proto: &dyn DeviceProtocol,
    mut payload: PushPayload,
    skip_verify: bool,
    check_in: &dyn Fn() -> bool,
) -> Result<PEQData> {
    let caps = profile.capabilities();
    let num_bands = caps.num_bands;
    let dsp_sample_rate = caps.dsp_sample_rate;
    payload.clamp(&caps);
    if let Err(e) = payload.is_valid(&caps) {
        return Err(AppError::new(ErrorKind::InvalidPayload, e));
    }

    wake_device(device, proto);
    let snapshot = pull_peq_data(device, proto, true, num_bands, check_in)?;

    if check_in() {
        return Err(AppError::new(ErrorKind::OperationCancelled, "Cancelled"));
    }

    if let Err(e) = do_write_and_commit(device, proto, &payload, num_bands, dsp_sample_rate) {
        rollback_and_verify(
            device,
            proto,
            &snapshot,
            num_bands,
            dsp_sample_rate,
            &caps,
            check_in,
        )
        .map_err(|r| {
            AppError::new(
                ErrorKind::RollbackFailed,
                format!(
                    "Write failed: {} | rollback failed: {}",
                    e.message, r.message
                ),
            )
        })?;
        return Err(e);
    }

    if skip_verify {
        return Ok(PEQData {
            filters: payload.filters.clone(),
            global_gain: payload.global_gain.unwrap_or(snapshot.global_gain),
        });
    }

    verify_or_rollback(
        device,
        proto,
        &payload.filters,
        payload.global_gain,
        &snapshot,
        num_bands,
        dsp_sample_rate,
        &caps,
        check_in,
    )
}

pub fn reset_with_verify(
    device: &dyn PhysicalInterface,
    profile: &dyn DeviceProfile,
    proto: &dyn DeviceProtocol,
    check_in: &dyn Fn() -> bool,
) -> Result<PEQData> {
    let caps = profile.capabilities();
    let num_bands = caps.num_bands;
    let dsp_sample_rate = caps.dsp_sample_rate;

    let packets = proto.build_reset_packets(num_bands, dsp_sample_rate);
    let timing = proto.write_timing();

    let framer_box = proto.framer();
    let framer = framer_box.as_ref();

    for packet in packets {
        if check_in() {
            return Err(AppError::new(ErrorKind::OperationCancelled, "Cancelled"));
        }
        send_report(device, &packet, framer)?;
        delay_ms(timing.per_filter_ms.max(timing.flood_delay_ms));
    }
    for packet in proto.build_commit_packets() {
        if check_in() {
            return Err(AppError::new(ErrorKind::OperationCancelled, "Cancelled"));
        }
        send_report(device, &packet, framer).map_err(|e| {
            AppError::new(
                ErrorKind::RollbackFailed,
                format!(
                    "Reset commit failed: {} — device may be in an inconsistent state",
                    e.message
                ),
            )
        })?;
    }

    pull_with_retry(device, proto, true, num_bands, check_in)
}
