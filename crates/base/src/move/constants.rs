use super::Move;

pub const NULL_MOVE: Move = Move::null_move();

/// From Square (0..63):
///  - Bits: 0000 0000 0011 1111
pub const FROM_SHIFT: u16 = 0x0000;
pub const FROM_MASK: u16 = 0x003F;

/// To Square (0..63):
///  - Bits: 0000 1111 1100 0000
pub const TO_SHIFT: u16 = 0x0006;
pub const TO_MASK: u16 = 0x003F;

/// Flag (0..15):
///  - Bits: 1111 0000 0000 0000
pub const FLAG_SHIFT: u16 = 0x0C;
pub const FLAG_MASK: u16 = 0xF000;

pub const PROMOTION_FLAG_MASK: u16 = 0b1000;
pub const PROMOTION_FLAG_SHIFT: u16 = 0x03;

pub const CAPTURE_FLAG_MASK: u16 = 0b0100;
pub const CAPTURE_FLAG_SHIFT: u16 = 0x02;
