use std::{fmt::Display, str::FromStr};

use crate::{
    board::{
        color::Color,
        piece::{ColoredPiece, Piece},
        Board,
    },
    square::{constants::*, Square},
};

use super::{
    constants::*,
    error::{InvalidMoveFormat, MoveError, PieceNotFound},
};

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct Move(u64);

impl Move {
    pub const fn new(
        piece: Piece,
        from: Square,
        to: Square,
        captured_piece: Piece,
        capture_square: Square,
        promoted_piece: Piece,
        is_castling: bool,
        is_en_passant: bool,
    ) -> Self {
        let mut bits = 0;

        bits |= (from.index() as u64) & SQUARE_MASK;
        bits |= ((to.index() as u64) & SQUARE_MASK) << TO_SHIFT;
        bits |= ((piece.index() as u64) & PIECE_MASK) << PIECE_SHIFT;
        bits |= (is_castling as u64) << IS_CASTLING_SHIFT;
        bits |= ((captured_piece.index() as u64) & PIECE_MASK) << CAPTURED_SHIFT;
        bits |= (is_en_passant as u64) << IS_EN_PASSANT_SHIFT;
        bits |= ((promoted_piece.index() as u64) & PIECE_MASK) << IS_PROMOTED_SHIFT;
        bits |= ((capture_square.index() as u64) & SQUARE_MASK) << CAPTURE_SQUARE_SHIFT;

        Self(bits)
    }

    /// Creates a quiet move like a pawn push or king move.
    ///
    /// ```rust
    /// use api::{
    ///     square::{constants::*, Square},
    ///     board::piece::Piece,
    ///     r#move::Move,
    /// };
    ///
    /// let mov = Move::quiet(Piece::Pawn, A2, A3);
    /// assert_eq!(mov.piece(), Piece::Pawn);
    /// assert_eq!(mov.from(), A2);
    /// assert_eq!(mov.to(), A3);
    /// assert_eq!(mov.captured_piece(), Piece::None);
    /// assert_eq!(mov.promoted_piece(), Piece::None);
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
        )
    }

    /// Creates a capture move.
    ///
    /// ```rust
    /// use api::{
    ///     square::{constants::*, Square},
    ///     board::piece::Piece,
    ///     r#move::Move,
    /// };
    ///
    /// let mov = Move::capture(Piece::Knight, C4, D6, Piece::Pawn);
    /// assert_eq!(mov.piece(), Piece::Knight);
    /// assert_eq!(mov.from(), C4);
    /// assert_eq!(mov.to(), D6);
    /// assert_eq!(mov.captured_piece(), Piece::Pawn);
    /// assert_eq!(mov.promoted_piece(), Piece::None);
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
        )
    }

    /// Creates a capture move.
    ///
    /// ```rust
    /// use api::{
    ///     square::{constants::*, Square},
    ///     board::piece::Piece,
    ///     r#move::Move,
    /// };
    ///
    /// let mov = Move::en_passant(D5, E6, E5);
    /// assert_eq!(mov.piece(), Piece::Pawn);
    /// assert_eq!(mov.from(), D5);
    /// assert_eq!(mov.to(), E6);
    /// assert_eq!(mov.captured_piece(), Piece::Pawn);
    /// assert_eq!(mov.promoted_piece(), Piece::None);
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
            true,
        )
    }

    /// Creates a capture move.
    ///
    /// ```rust
    /// use api::{
    ///     square::{constants::*, Square},
    ///     board::piece::Piece,
    ///     r#move::Move,
    /// };
    ///
    /// let mov = Move::promotion(D7, D8, Piece::Queen, Piece::Rook);
    /// assert_eq!(mov.piece(), Piece::Pawn);
    /// assert_eq!(mov.from(), D7);
    /// assert_eq!(mov.to(), D8);
    /// assert_eq!(mov.captured_piece(), Piece::Rook);
    /// assert_eq!(mov.promoted_piece(), Piece::Queen);
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
        )
    }

    /// Creates a capture move.
    ///
    /// ```rust
    /// use api::{
    ///     square::{constants::*, Square},
    ///     board::piece::Piece,
    ///     r#move::Move,
    /// };
    ///
    /// let mov = Move::castling(E1, G1);
    /// assert_eq!(mov.piece(), Piece::King);
    /// assert_eq!(mov.from(), E1);
    /// assert_eq!(mov.to(), G1);
    /// assert_eq!(mov.captured_piece(), Piece::None);
    /// assert_eq!(mov.promoted_piece(), Piece::None);
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
            true,
            false,
        )
    }

    #[inline(always)]
    pub const fn bits(&self) -> u64 {
        self.0
    }
}

impl Move {
    #[inline(always)]
    pub const fn from(&self) -> Square {
        let index = self.0 & SQUARE_MASK;
        Square::from_index(index as u8)
    }

    #[inline(always)]
    pub const fn to(&self) -> Square {
        let index = (self.0 >> TO_SHIFT) & SQUARE_MASK;
        Square::from_index(index as u8)
    }

    #[inline(always)]
    pub const fn piece(&self) -> Piece {
        let index = (self.0 >> PIECE_SHIFT) & PIECE_MASK;
        Piece::from_index(index as usize)
    }

    #[inline(always)]
    pub fn is_double_pawn(&self) -> bool {
        if self.captured_piece() != Piece::None {
            return false;
        }
        if self.piece() != Piece::Pawn {
            return false;
        }

        let from = self.from();
        let to = self.to();

        let square_difference = (isize::from(to) - isize::from(from)).abs();
        square_difference == 16
    }

    #[inline(always)]
    pub const fn is_castling(&self) -> bool {
        (self.0 & IS_CASTLING_MASK) != 0
    }

    #[inline(always)]
    pub const fn captured_piece(&self) -> Piece {
        let index = (self.0 >> CAPTURED_SHIFT) & PIECE_MASK;
        Piece::from_index(index as usize)
    }

    #[inline(always)]
    pub const fn capture_square(&self) -> Square {
        let index = (self.0 >> CAPTURE_SQUARE_SHIFT) & SQUARE_MASK;
        Square::from_index(index as u8)
    }

    #[inline(always)]
    pub const fn is_en_passant(&self) -> bool {
        (self.0 & IS_EN_PASSANT_MASK) != 0
    }

    #[inline(always)]
    pub const fn promoted_piece(&self) -> Piece {
        let index = (self.0 >> IS_PROMOTED_SHIFT) & PIECE_MASK;
        Piece::from_index(index as usize)
    }

    #[inline(always)]
    pub const fn is_quiet(&self) -> bool {
        (self.0 & IS_QUIET_MASK) == 0
    }

    #[inline(always)]
    pub const fn is_capture(&self) -> bool {
        (self.0 & IS_CAPTURE_MASK) != 0
    }

    pub const fn is_promotion(&self) -> bool {
        (self.0 & IS_PROMOTED_MASK) != 0
    }

    #[inline(always)]
    pub fn is_tactical(&self) -> bool {
        self.is_capture() || self.is_promotion()
    }
}

impl Move {
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

        let is_en_passant = match board.en_passant() {
            Some(en_passant) => en_passant.to_move == to && colored_piece.piece == Piece::Pawn,
            _ => false,
        };

        let mov = if is_en_passant {
            let en_passant = board.en_passant().as_ref().unwrap();
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
            Move::quiet(colored_piece.piece, from, to)
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
