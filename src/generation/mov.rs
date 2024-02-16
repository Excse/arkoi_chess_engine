use std::{fmt::Display, str::FromStr};

use crate::{
    bitboard::{constants::*, square::Square},
    board::{
        color::Color,
        piece::{ColoredPiece, Piece},
        Board,
    },
};

use super::error::{InvalidMoveFormat, MoveError, PieceNotFound};

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

/// Is Double Pawn (0..1):
///  - Bits: 0000 0000 0000 0000 1000 0000 0000 0000
pub const IS_DOUBLE_PAWN_SHIFT: u64 = 0x0F;
pub const IS_DOUBLE_PAWN_MASK: u64 = 0x8000;

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

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct Move {
    pub bits: u64,
}

impl Move {
    pub fn new(
        piece: Piece,
        from: Square,
        to: Square,
        captured_piece: Piece,
        capture_square: Square,
        promoted_piece: Piece,
        is_double_pawn: bool,
        is_castling: bool,
        is_en_passant: bool,
    ) -> Self {
        let mut bits = 0;

        bits |= u64::from(from) & SQUARE_MASK;
        bits |= (u64::from(to) & SQUARE_MASK) << TO_SHIFT;
        bits |= ((piece.index() as u64) & PIECE_MASK) << PIECE_SHIFT;
        bits |= (is_double_pawn as u64) << IS_DOUBLE_PAWN_SHIFT;
        bits |= (is_castling as u64) << IS_CASTLING_SHIFT;
        bits |= ((captured_piece.index() as u64) & PIECE_MASK) << CAPTURED_SHIFT;
        bits |= (is_en_passant as u64) << IS_EN_PASSANT_SHIFT;
        bits |= ((promoted_piece.index() as u64) & PIECE_MASK) << IS_PROMOTED_SHIFT;
        bits |= (u64::from(capture_square) & SQUARE_MASK) << CAPTURE_SQUARE_SHIFT;

        Self { bits }
    }

    /// Creates a quiet move like a pawn push or king move.
    ///
    /// ```rust
    /// let mov = Move::quiet(Piece::Pawn, Square::A2, Square::A3);
    /// assert_eq!(mov.piece(), Piece::Pawn);
    /// assert_eq!(mov.from(), Square::A2);
    /// assert_eq!(mov.to(), Square::A3);
    /// assert_eq!(mov.captured(), Piece::None);
    /// assert_eq!(mov.promoted(), Piece::None);
    /// assert_eq!(mov.is_double_pawn(), false);
    /// assert_eq!(mov.is_castling(), false);
    /// assert_eq!(mov.is_en_passant(), false);
    /// assert_eq!(mov.is_quiet(), true);
    /// assert_eq!(mov.is_capture(), false);
    /// assert_eq!(mov.is_promotion(), false);
    /// ```
    pub fn quiet(piece: Piece, from: Square, to: Square) -> Self {
        Self::new(
            piece,
            from,
            to,
            Piece::None,
            Square::default(),
            Piece::None,
            false,
            false,
            false,
        )
    }

    /// Creates a double pawn move.
    ///
    /// ```rust
    /// let mov = Move::double_pawn(Piece::Pawn, Square::A2, Square::A4);
    /// assert_eq!(mov.piece(), Piece::Pawn);
    /// assert_eq!(mov.from(), Square::A2);
    /// assert_eq!(mov.to(), Square::A4);
    /// assert_eq!(mov.captured(), Piece::None);
    /// assert_eq!(mov.promoted(), Piece::None);
    /// assert_eq!(mov.is_double_pawn(), true);
    /// assert_eq!(mov.is_castling(), false);
    /// assert_eq!(mov.is_en_passant(), false);
    /// assert_eq!(mov.is_quiet(), true);
    /// assert_eq!(mov.is_capture(), false);
    /// assert_eq!(mov.is_promotion(), false);
    /// ```
    pub fn double_pawn(from: Square, to: Square) -> Self {
        Self::new(
            Piece::Pawn,
            from,
            to,
            Piece::None,
            Square::default(),
            Piece::None,
            true,
            false,
            false,
        )
    }

    /// Creates a capture move.
    ///
    /// ```rust
    /// let mov = Move::capture(Piece::Knight, Square::C4, Square::D6, Piece::Pawn);
    /// assert_eq!(mov.piece(), Piece::Knight);
    /// assert_eq!(mov.from(), Square::C4);
    /// assert_eq!(mov.to(), Square::D6);
    /// assert_eq!(mov.captured(), Piece::Pawn);
    /// assert_eq!(mov.promoted(), Piece::None);
    /// assert_eq!(mov.is_double_pawn(), false);
    /// assert_eq!(mov.is_castling(), false);
    /// assert_eq!(mov.is_en_passant(), false);
    /// assert_eq!(mov.is_quiet(), false);
    /// assert_eq!(mov.is_capture(), true);
    /// assert_eq!(mov.is_promotion(), false);
    /// ```
    pub fn capture(piece: Piece, from: Square, to: Square, captured_piece: Piece) -> Self {
        Self::new(
            piece,
            from,
            to,
            captured_piece,
            to,
            Piece::None,
            false,
            false,
            false,
        )
    }

    /// Creates a capture move.
    ///
    /// ```rust
    /// let mov = Move::en_passant(Square::D5, Square::E6);
    /// assert_eq!(mov.piece(), Piece::Pawn);
    /// assert_eq!(mov.from(), Square::D5);
    /// assert_eq!(mov.to(), Square::E6);
    /// assert_eq!(mov.captured(), Piece::Pawn);
    /// assert_eq!(mov.promoted(), Piece::None);
    /// assert_eq!(mov.is_double_pawn(), false);
    /// assert_eq!(mov.is_castling(), false);
    /// assert_eq!(mov.is_en_passant(), true);
    /// assert_eq!(mov.is_quiet(), false);
    /// assert_eq!(mov.is_capture(), true);
    /// assert_eq!(mov.is_promotion(), false);
    /// ```
    pub fn en_passant(from: Square, to: Square, capture_square: Square) -> Self {
        Self::new(
            Piece::Pawn,
            from,
            to,
            Piece::Pawn,
            capture_square,
            Piece::None,
            false,
            false,
            true,
        )
    }

    /// Creates a capture move.
    ///
    /// ```rust
    /// let mov = Move::promotion(Square::D7, Square::D8, Piece::Queen, Piece::Rook);
    /// assert_eq!(mov.piece(), Piece::Pawn);
    /// assert_eq!(mov.from(), Square::D7);
    /// assert_eq!(mov.to(), Square::D8);
    /// assert_eq!(mov.captured(), Piece::Rook);
    /// assert_eq!(mov.promoted(), Piece::Queen);
    /// assert_eq!(mov.is_double_pawn(), false);
    /// assert_eq!(mov.is_castling(), false);
    /// assert_eq!(mov.is_en_passant(), false);
    /// assert_eq!(mov.is_quiet(), false);
    /// assert_eq!(mov.is_capture(), true);
    /// assert_eq!(mov.is_promotion(), true);
    /// ```
    pub fn promotion(
        from: Square,
        to: Square,
        promoted_piece: Piece,
        captured_piece: Piece,
    ) -> Self {
        let capture_square = match captured_piece {
            Piece::None => Square::default(),
            _ => to,
        };
        Self::new(
            Piece::Pawn,
            from,
            to,
            captured_piece,
            capture_square,
            promoted_piece,
            false,
            false,
            false,
        )
    }

    /// Creates a capture move.
    ///
    /// ```rust
    /// let mov = Move::castling(Square::E1, Square::G1);
    /// assert_eq!(mov.piece(), Piece::King);
    /// assert_eq!(mov.from(), Square::E1);
    /// assert_eq!(mov.to(), Square::G1);
    /// assert_eq!(mov.captured(), Piece::None);
    /// assert_eq!(mov.promoted(), Piece::None);
    /// assert_eq!(mov.is_double_pawn(), false);
    /// assert_eq!(mov.is_castling(), true);
    /// assert_eq!(mov.is_en_passant(), false);
    /// assert_eq!(mov.is_quiet(), true);
    /// assert_eq!(mov.is_capture(), false);
    /// assert_eq!(mov.is_promotion(), false);
    /// ```
    pub fn castling(from: Square, to: Square) -> Self {
        Self::new(
            Piece::King,
            from,
            to,
            Piece::None,
            Square::default(),
            Piece::None,
            false,
            true,
            false,
        )
    }

    #[inline(always)]
    pub const fn from(&self) -> Square {
        let index = self.bits & SQUARE_MASK;
        Square::by_index(index as u8)
    }

    #[inline(always)]
    pub const fn to(&self) -> Square {
        let index = (self.bits >> TO_SHIFT) & SQUARE_MASK;
        Square::by_index(index as u8)
    }

    #[inline(always)]
    pub const fn piece(&self) -> Piece {
        let index = (self.bits >> PIECE_SHIFT) & PIECE_MASK;
        Piece::from_index(index as usize)
    }

    #[inline(always)]
    pub const fn is_double_pawn(&self) -> bool {
        (self.bits & IS_DOUBLE_PAWN_MASK) != 0
    }

    #[inline(always)]
    pub const fn is_castling(&self) -> bool {
        (self.bits & IS_CASTLING_MASK) != 0
    }

    #[inline(always)]
    pub const fn captured_piece(&self) -> Piece {
        let index = (self.bits >> CAPTURED_SHIFT) & PIECE_MASK;
        Piece::from_index(index as usize)
    }

    #[inline(always)]
    pub const fn capture_square(&self) -> Square {
        let index = (self.bits >> CAPTURE_SQUARE_SHIFT) & SQUARE_MASK;
        Square::by_index(index as u8)
    }

    #[inline(always)]
    pub const fn is_en_passant(&self) -> bool {
        (self.bits & IS_EN_PASSANT_MASK) != 0
    }

    #[inline(always)]
    pub const fn promoted_piece(&self) -> Piece {
        let index = (self.bits >> IS_PROMOTED_SHIFT) & PIECE_MASK;
        Piece::from_index(index as usize)
    }

    #[inline(always)]
    pub const fn is_quiet(&self) -> bool {
        (self.bits & IS_QUIET_MASK) == 0
    }

    #[inline(always)]
    pub const fn is_capture(&self) -> bool {
        (self.bits & IS_CAPTURE_MASK) != 0
    }

    pub const fn is_promotion(&self) -> bool {
        (self.bits & IS_PROMOTED_MASK) != 0
    }

    #[inline(always)]
    pub fn is_tactical(&self) -> bool {
        self.is_capture() || self.is_promotion()
    }

    pub fn parse(board: &Board, input: impl Into<String>) -> Result<Self, MoveError> {
        let input = input.into();

        if input.len() < 4 {
            return Err(InvalidMoveFormat::new(input.clone()).into());
        }

        let from = Square::from_str(&input[0..2])?;
        let to = Square::from_str(&input[2..4])?;

        let colored_piece = board
            .get_piece_type(from)
            .ok_or(PieceNotFound::new(from.to_string()))?;
        let captured = match board.get_piece_type(to) {
            Some(colored_piece) => colored_piece.piece,
            _ => Piece::None,
        };

        let promoted_piece = match input.len() {
            5 => {
                let piece = input
                    .chars()
                    .nth(4)
                    .ok_or(InvalidMoveFormat::new(input.clone()))?;
                let colored_piece = ColoredPiece::from_fen(piece)?;
                Some(colored_piece.piece)
            }
            _ => None,
        };

        let is_en_passant = match &board.gamestate.en_passant {
            Some(en_passant) => en_passant.to_move == to && colored_piece.piece == Piece::Pawn,
            _ => false,
        };

        let mov = if is_en_passant {
            let en_passant = board.gamestate.en_passant.as_ref().unwrap();
            Move::en_passant(from, to, en_passant.to_capture)
        } else if let Some(promoted) = promoted_piece {
            Move::promotion(from, to, promoted, captured)
        } else if captured != Piece::None {
            Move::capture(colored_piece.piece, from, to, captured)
        } else if colored_piece.piece == Piece::King {
            match (colored_piece.color, from, to) {
                (Color::Black, E8, G8)
                | (Color::Black, E8, C8)
                | (Color::White, E1, G1)
                | (Color::White, E1, C1) => Move::castling(from, to),
                _ => Move::quiet(Piece::King, from, to),
            }
        } else {
            let square_difference = (isize::from(to) - isize::from(from)).abs();
            if captured == Piece::None
                && colored_piece.piece == Piece::Pawn
                && square_difference == 16
            {
                Move::double_pawn(from, to)
            } else {
                Move::quiet(colored_piece.piece, from, to)
            }
        };

        return Ok(mov);
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let from = self.from().to_string();
        let to = self.to().to_string();

        if self.is_promotion() {
            let colored_piece = ColoredPiece::new(self.promoted_piece(), Color::Black);
            write!(f, "{}{}{}", from, to, colored_piece.to_fen())
        } else {
            write!(f, "{}{}", from, to)
        }
    }
}
