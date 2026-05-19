// Copyright (c) 2026 Bukutsu
// SPDX-License-Identifier: MIT

pub const REPORT_ID: u8 = 0x4B;

pub const CMD_FLASH_EQ: u8 = 0x01;
pub const CMD_GLOBAL_GAIN: u8 = 0x03;
pub const CMD_PEQ_VALUES: u8 = 0x09;
pub const CMD_TEMP_WRITE: u8 = 0x0A;
pub const CMD_VERSION: u8 = 0x0C;

pub const READ: u8 = 0x80;
pub const WRITE: u8 = 0x01;
pub const END: u8 = 0x00;

pub const FILTER_SLOT: u8 = 101;

pub const OFFSET_CMD_TYPE: usize = 0;
pub const OFFSET_CMD: usize = 1;
pub const OFFSET_NONCE: usize = 2;
pub const OFFSET_INDEX: usize = 4;
pub const OFFSET_BIQUAD_START: usize = 7;
pub const OFFSET_FREQ_L: usize = 27;
pub const OFFSET_FREQ_H: usize = 28;
pub const OFFSET_Q_L: usize = 29;
pub const OFFSET_Q_H: usize = 30;
pub const OFFSET_GAIN_L: usize = 31;
pub const OFFSET_GAIN_H: usize = 32;
pub const OFFSET_FILTER_TYPE: usize = 33;
pub const OFFSET_SLOT: usize = 35;

pub const OFFSET_GAIN_VALUE: usize = 4;

#[derive(Debug, Clone)]
pub struct ReadTiming {
    pub post_version_ms: u64,
    pub filter_request_ms: u64,
    pub inter_filter_ms: u64,
    pub post_filter_read_ms: u64,
    pub post_global_gain_ms: u64,
    pub read_timeout_ms: u32,
}

impl Default for ReadTiming {
    fn default() -> Self {
        Self {
            post_version_ms: 50,
            filter_request_ms: 10,
            inter_filter_ms: 10,
            post_filter_read_ms: 40,
            post_global_gain_ms: 25,
            read_timeout_ms: 60,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WriteTiming {
    pub per_filter_ms: u64,
    pub flood_delay_ms: u64,
    pub batch_ms: u64,
    pub global_gain_ms: u64,
    pub commit_ms: u64,
}

impl Default for WriteTiming {
    fn default() -> Self {
        Self {
            per_filter_ms: 80,
            flood_delay_ms: 5,
            batch_ms: 100,
            global_gain_ms: 50,
            commit_ms: 500,
        }
    }
}
