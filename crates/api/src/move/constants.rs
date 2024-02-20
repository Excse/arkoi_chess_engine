use crate::{board::piece::Piece, square::constants::*};

use super::Move;

pub const NULL_MOVE: Move = Move::new(
    Piece::None,
    A1,
    A1,
    Piece::None,
    A1,
    Piece::None,
    false,
    false,
);
const _: () = assert!(NULL_MOVE.bits() == 0);

/// From Square (0..63):
///  - Bits: 0000 0000 0000 0000 0000 0000 0011 1111
pub const SQUARE_MASK: u64 = 0x3F;

/// To Square (0..63):
///  - Bits: 0000 0000 0000 0000 0000 1111 1100 0000
pub const TO_SHIFT: u64 = 0x06;

/// Moving Piece (0..7):
///  - Bits: 0000 0000 0000 0000 0111 0000 0000 0000
pub const PIECE_SHIFT: u64 = 0x0C;
pub const PIECE_MASK: u64 = 0x07;

/// Is Castling (0..1):
///  - Bits: 0000 0000 0000 0001 0000 0000 0000 0000
pub const IS_CASTLING_SHIFT: u64 = 0x10;
pub const IS_CASTLING_MASK: u64 = 0x10000;

/// Captured Piece (0..7):
///  - Bits: 0000 0000 0000 1110 0000 0000 0000 0000
pub const CAPTURED_SHIFT: u64 = 0x11;

/// Is En Passant (0..1):
///  - Bits: 0000 0000 0001 0000 0000 0000 0000 0000
pub const IS_EN_PASSANT_SHIFT: u64 = 0x14;
pub const IS_EN_PASSANT_MASK: u64 = 0x100000;

/// Promoted Piece (0..7):
///  - Bits: 0000 0000 1110 0000 0000 0000 0000 0000
pub const IS_PROMOTED_SHIFT: u64 = 0x15;
pub const IS_PROMOTED_MASK: u64 = 0xE00000;

/// Capture Square (0..63):
///  - Bits: 0011 1111 0000 0000 0000 0000 0000 0000
pub const CAPTURE_SQUARE_SHIFT: u64 = 0x18;

/// Is Quiet (no capture, no promotion, no en passant):
///  - Bits: 0000 0000 1111 1110 0000 0000 0000 0000
///  - Condition: Must be 0
pub const IS_QUIET_MASK: u64 = 0xFE0000;

/// Is Capture (captured piece or en passant):
///  - Condition: Must be not 0
///  - Bits: 0000 0000 0001 1110 0000 0000 0000 0000
pub const IS_CAPTURE_MASK: u64 = 0x1E0000;
