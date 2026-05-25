// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

//! Re-exports of TP35 Pro wire-format constants from `core/device/tp35pro` and
//! timing structs from `core/device/timing`. All definitions live in `core/` so
//! device impls remain independent of the hardware layer.

pub use crate::core::device::timing::{ReadTiming, WriteTiming};
pub use crate::hardware::devices::tp35pro::{
    CMD_FLASH_EQ, CMD_GLOBAL_GAIN, CMD_PEQ_VALUES, CMD_TEMP_WRITE, CMD_VERSION, CONST_FLASH_EQ_LEN,
    CONST_GLOBAL_GAIN_LEN, CONST_PEQ_PAYLOAD_LEN, CONST_TEMP_WRITE_LEN, CONST_TEMP_WRITE_MAGIC_A,
    CONST_TEMP_WRITE_MAGIC_B, END, FILTER_SLOT, OFFSET_BIQUAD_START, OFFSET_CMD, OFFSET_CMD_TYPE,
    OFFSET_FILTER_TYPE, OFFSET_FREQ_H, OFFSET_FREQ_L, OFFSET_GAIN_H, OFFSET_GAIN_L,
    OFFSET_GAIN_VALUE, OFFSET_INDEX, OFFSET_NONCE, OFFSET_Q_H, OFFSET_Q_L, OFFSET_SLOT, READ,
    REPORT_ID, WRITE,
};
