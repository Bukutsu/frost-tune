use crate::error::{AppError, ErrorKind, Result};
use crate::hardware::hid::delay_ms;
use crate::hardware::operations::{compare_peq, pull_peq_data, rollback_and_verify};
use crate::hardware::packet_builder::{
    commit_changes, init_device_session, write_filters_and_gain,
};
use crate::hardware::protocol::DeviceProtocol;
use crate::models::{PEQData, PushPayload};

pub fn pull_with_retry(
    device: &hidapi::HidDevice,
    proto: &dyn DeviceProtocol,
    strict: bool,
) -> Result<PEQData> {
    let wake_request = proto.build_global_gain_request(0x01);
    let _ = crate::hardware::hid::send_report(device, &wake_request[..], proto.report_id());
    delay_ms(50);
    let first_result = pull_peq_data(device, proto, strict);

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
        delay_ms(100);
        match pull_peq_data(device, proto, strict) {
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

pub fn push_with_verify(
    device: &hidapi::HidDevice,
    proto: &dyn DeviceProtocol,
    mut payload: PushPayload,
) -> Result<PEQData> {
    payload.clamp();
    payload
        .is_valid()
        .map_err(|e| AppError::new(ErrorKind::ParseError, e))?;

    let wake_request = proto.build_global_gain_request(0x01);
    let _ = crate::hardware::hid::send_report(device, &wake_request[..], proto.report_id());
    delay_ms(50);
    let snapshot = pull_peq_data(device, proto, true)?;

    let timing = proto.write_timing();
    let write_res = (|| -> Result<()> {
        init_device_session(device, proto)?;
        write_filters_and_gain(
            device,
            proto,
            &payload.filters,
            payload.global_gain.unwrap_or(0),
            &timing,
        )?;
        commit_changes(device, proto, &timing)?;
        Ok(())
    })();

    if let Err(e) = write_res {
        if let Err(rollback_error) = rollback_and_verify(device, proto, &snapshot) {
            return Err(AppError::new(
                ErrorKind::RollbackFailed,
                format!(
                    "Write failed: {} | rollback failed: {}",
                    e.message, rollback_error.message
                ),
            ));
        }
        return Err(e);
    }

    for attempt in 0..3 {
        let backoff_ms = 200 * (2u64.pow(attempt as u32));
        delay_ms(backoff_ms as u64);
        match pull_peq_data(device, proto, true) {
            Ok(read_back) => {
                if compare_peq(
                    &read_back,
                    &payload.filters,
                    payload.global_gain.unwrap_or(0),
                )
                .is_ok()
                {
                    return Ok(read_back);
                }
            }
            Err(e) => {
                if let Err(rollback_error) = rollback_and_verify(device, proto, &snapshot) {
                    return Err(AppError::new(
                        ErrorKind::RollbackFailed,
                        format!(
                            "Verify read error: {} | rollback failed: {}",
                            e.message, rollback_error.message
                        ),
                    ));
                }
                return Err(e);
            }
        }
    }

    if let Err(rollback_error) = rollback_and_verify(device, proto, &snapshot) {
        return Err(AppError::new(
            ErrorKind::RollbackFailed,
            format!(
                "Verification failed: settings did not match | rollback failed: {}",
                rollback_error.message
            ),
        ));
    }

    Err(AppError::new(
        ErrorKind::VerifyFailed,
        "Verification failed: settings did not match",
    ))
}
