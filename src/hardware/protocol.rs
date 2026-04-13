pub const REPORT_ID: u8 = 0x4B;

pub const CMD_FLASH_EQ: u8 = 0x01;
pub const CMD_GLOBAL_GAIN: u8 = 0x03;
pub const CMD_PEQ_VALUES: u8 = 0x09;
pub const CMD_TEMP_WRITE: u8 = 0x0A;
pub const CMD_VERSION: u8 = 0x0C;

pub const READ: u8 = 0x80;
pub const WRITE: u8 = 0x01;
pub const END: u8 = 0x00;

// Packet offsets for Filter data (assuming no REPORT_ID offset in internal logic)
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
pub const OFFSET_SLOT: usize = 35; // Slot byte in write packet

// Global Gain offsets
pub const OFFSET_GAIN_VALUE: usize = 4;
