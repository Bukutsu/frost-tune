use crate::error::{AppError, ErrorKind, Result};
use crate::hardware::hid::{delay_ms, pull_peq_internal};
use crate::hardware::packet_builder::{
    commit_changes, write_filters_and_gain,
};
use crate::hardware::protocol::DeviceProtocol;
use crate::models::{Filter, PEQData};

pub fn pull_peq_data(
    d: &hidapi::HidDevice,
    proto: &dyn DeviceProtocol,
    strict: bool,
) -> Result<PEQData> {
    let mut last_err = AppError::new(ErrorKind::ReadTimeout, "Timeout");
    for attempt in 0..3 {
        if attempt > 0 {
            log::info!("Retrying PEQ pull, attempt {}...", attempt + 1);
        }
        crate::hardware::hid::flush_hid_buffer(d);
        match pull_peq_internal(d, proto, strict) {
            Ok(data) => return Ok(data),
            Err(e) => {
                log::warn!("PEQ pull attempt {} failed: {}", attempt + 1, e.message);
                last_err = e;
            }
        }
        if attempt < 2 {
            delay_ms(200);
        }
    }
    Err(last_err)
}

pub fn rollback_state(
    d: &hidapi::HidDevice,
    proto: &dyn DeviceProtocol,
    state: &PEQData,
) -> Result<()> {
    let timing = proto.write_timing();
    write_filters_and_gain(d, proto, &state.filters, state.global_gain, &timing)?;
    commit_changes(d, proto, &timing)
}

pub fn rollback_and_verify(
    d: &hidapi::HidDevice,
    proto: &dyn DeviceProtocol,
    snapshot: &PEQData,
) -> Result<()> {
    log::info!("Starting hardware state rollback and verification...");
    rollback_state(d, proto, snapshot).map_err(|e| {
        let msg = format!("rollback write failed: {}", e.message);
        log::error!("{}", msg);
        AppError::new(ErrorKind::RollbackFailed, msg)
    })?;

    let restored = pull_peq_data(d, proto, true).map_err(|e| {
        let msg = format!("rollback verify read failed: {}", e.message);
        log::error!("{}", msg);
        AppError::new(ErrorKind::RollbackFailed, msg)
    })?;

    compare_peq(&restored, &snapshot.filters, snapshot.global_gain).map_err(|e| {
        let msg = format!("rollback verify mismatch: {}", e.message);
        log::error!("{}", msg);
        AppError::new(ErrorKind::RollbackFailed, msg)
    })?;

    log::info!("Hardware state rollback successful and verified.");
    Ok(())
}

pub fn compare_peq(actual: &PEQData, filters: &[Filter], gain: i8) -> Result<()> {
    if actual.global_gain != gain {
        return Err(AppError::new(
            ErrorKind::VerifyFailed,
            format!(
                "Global gain mismatch: expected {}, got {}",
                gain, actual.global_gain
            ),
        ));
    }
    for (a, f) in actual.filters.iter().zip(filters.iter()) {
        if (a.gain - f.gain).abs() > 0.15 {
            return Err(AppError::new(
                ErrorKind::VerifyFailed,
                format!(
                    "Band {} gain mismatch: expected {:.2}, got {:.2}",
                    f.index, f.gain, a.gain
                ),
            ));
        }
        if (a.freq as i32 - f.freq as i32).abs() > 1 {
            return Err(AppError::new(
                ErrorKind::VerifyFailed,
                format!(
                    "Band {} freq mismatch: expected {}, got {}",
                    f.index, f.freq, a.freq
                ),
            ));
        }
        if (a.q - f.q).abs() > 0.05 {
            return Err(AppError::new(
                ErrorKind::VerifyFailed,
                format!(
                    "Band {} Q mismatch: expected {:.2}, got {:.2}",
                    f.index, f.q, a.q
                ),
            ));
        }
        if f.filter_type != a.filter_type {
            return Err(AppError::new(
                ErrorKind::VerifyFailed,
                format!("Band {} filter type mismatch", f.index),
            ));
        }
    }
    Ok(())
}
