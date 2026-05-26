// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

use crate::core::device::protocol::DeviceProtocol;
use crate::core::{Filter, PEQData, PushPayload};
use crate::error::{AppError, ErrorKind, Result};
use crate::hardware::hid::{delay_ms, send_report, HidDeviceIo};
use crate::hardware::operations::{compare_peq, pull_peq_data, rollback_and_verify};
use crate::hardware::packet_builder::{
    commit_changes, init_device_session, write_filters_and_gain,
};
use crate::hardware::DeviceProfile;

pub fn pull_with_retry(
    device: &dyn HidDeviceIo,
    proto: &dyn DeviceProtocol,
    strict: bool,
    num_bands: usize,
    check_in: &dyn Fn() -> bool,
) -> Result<PEQData> {
    if check_in() {
        return Err(AppError::new(ErrorKind::OperationCancelled, "Cancelled"));
    }

    let wake_request = proto.build_global_gain_request(WAKE_NONCE);
    if let Err(e) = crate::hardware::hid::send_report(device, &wake_request[..], proto.report_id())
    {
        log::warn!("pull wake request failed: {}", e);
    }
    delay_ms(WAKE_DELAY_MS);
    let first_result = pull_peq_data(device, proto, strict, num_bands, check_in);

    let needs_retry = match &first_result {
        Ok(peq) => {
            let all_disabled = peq.filters.iter().all(|f| !f.enabled);
            let has_default_gain = peq.global_gain == 0;
            let all_default_freq = peq.filters.iter().all(|f| f.freq == 100);
            all_disabled && has_default_gain && all_default_freq
        }
        Err(_) => true,
    };

    if needs_retry {
        if check_in() {
            return Err(AppError::new(ErrorKind::OperationCancelled, "Cancelled"));
        }
        delay_ms(RETRY_DELAY_PULL_MS);
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
const WAKE_DELAY_MS: u64 = 50;
const RETRY_DELAY_PULL_MS: u64 = 100;
const VERIFY_RETRY_COUNT: usize = 3;
const VERIFY_BACKOFF_BASE_MS: u64 = 200;

fn wake_device(device: &dyn HidDeviceIo, proto: &dyn DeviceProtocol) {
    let wake = proto.build_global_gain_request(WAKE_NONCE);
    if let Err(e) = send_report(device, &wake[..], proto.report_id()) {
        log::warn!("wake request failed: {}", e);
    }
    delay_ms(WAKE_DELAY_MS);
}

fn do_write_and_commit(
    device: &dyn HidDeviceIo,
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

fn verify_or_rollback(
    device: &dyn HidDeviceIo,
    proto: &dyn DeviceProtocol,
    expected_filters: &[Filter],
    expected_gain: Option<i8>,
    snapshot: &PEQData,
    num_bands: usize,
    dsp_sample_rate: f64,
    check_in: &dyn Fn() -> bool,
) -> Result<PEQData> {
    for attempt in 0..VERIFY_RETRY_COUNT {
        let backoff_ms = VERIFY_BACKOFF_BASE_MS.saturating_mul(2u64.saturating_pow(attempt as u32));
        delay_ms(backoff_ms);
        match pull_peq_data(device, proto, true, num_bands, check_in) {
            Ok(read_back) => {
                if compare_peq(&read_back, expected_filters, expected_gain.unwrap_or(0)).is_ok() {
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
    rollback_and_verify(device, proto, snapshot, num_bands, dsp_sample_rate, check_in).map_err(
        |r| {
            AppError::new(
                ErrorKind::RollbackFailed,
                format!(
                    "Verification failed: settings did not match | rollback failed: {}",
                    r.message
                ),
            )
        },
    )?;
    Err(AppError::new(
        ErrorKind::VerifyFailed,
        "Verification failed: settings did not match",
    ))
}
pub fn push_with_verify(
    device: &dyn HidDeviceIo,
    profile: &dyn DeviceProfile,
    proto: &dyn DeviceProtocol,
    mut payload: PushPayload,
    check_in: &dyn Fn() -> bool,
) -> Result<PEQData> {
    let caps = profile.capabilities();
    let num_bands = caps.num_bands;
    let dsp_sample_rate = caps.dsp_sample_rate;
    payload.clamp(
        caps.freq_range,
        caps.band_gain_range,
        caps.q_range,
        caps.global_gain_range,
    );
    payload
        .is_valid(
            num_bands,
            caps.freq_range,
            caps.band_gain_range,
            caps.q_range,
            caps.global_gain_range,
        )
        .map_err(|e| AppError::new(ErrorKind::ParseError, e))?;

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

    verify_or_rollback(
        device,
        proto,
        &payload.filters,
        payload.global_gain,
        &snapshot,
        num_bands,
        dsp_sample_rate,
        check_in,
    )
}
