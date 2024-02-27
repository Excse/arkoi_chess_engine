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
    flag::MoveFlag,
};

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct Move(u16);

impl Move {
    pub fn from_bits(bits: u16) -> Self {
        Self(bits)
    }

    pub fn quiet(from: Square, to: Square) -> Self {
        let flag = MoveFlag::Quiet;
        let mut bits = 0;

        bits |= ((from.index() as u16) & FROM_MASK) << FROM_SHIFT;
        bits |= ((to.index() as u16) & TO_MASK) << TO_SHIFT;
        bits |= ((flag as u16) & FLAG_MASK) << FLAG_SHIFT;

        Self(bits)
    }

    pub fn double_pawn(from: Square, to: Square) -> Self {
        let flag = MoveFlag::DoublePawn;
        let mut bits = 0;

        bits |= ((from.index() as u16) & FROM_MASK) << FROM_SHIFT;
        bits |= ((to.index() as u16) & TO_MASK) << TO_SHIFT;
        bits |= ((flag as u16) & FLAG_MASK) << FLAG_SHIFT;

        Self(bits)
    }

    pub fn capture(from: Square, to: Square) -> Self {
        let flag = MoveFlag::Capture;
        let mut bits = 0;

        bits |= ((from.index() as u16) & FROM_MASK) << FROM_SHIFT;
        bits |= ((to.index() as u16) & TO_MASK) << TO_SHIFT;
        bits |= ((flag as u16) & FLAG_MASK) << FLAG_SHIFT;

        Self(bits)
    }

    pub fn en_passant(from: Square, to: Square) -> Self {
        let flag = MoveFlag::EnPassant;
        let mut bits = 0;

        bits |= ((from.index() as u16) & FROM_MASK) << FROM_SHIFT;
        bits |= ((to.index() as u16) & TO_MASK) << TO_SHIFT;
        bits |= ((flag as u16) & FLAG_MASK) << FLAG_SHIFT;

        Self(bits)
    }

    pub fn castle(from: Square, to: Square, kingside: bool) -> Self {
        let flag = if kingside {
            MoveFlag::KingCastle
        } else {
            MoveFlag::QueenCastle
        };
        let mut bits = 0;

        bits |= ((from.index() as u16) & FROM_MASK) << FROM_SHIFT;
        bits |= ((to.index() as u16) & TO_MASK) << TO_SHIFT;
        bits |= ((flag as u16) & FLAG_MASK) << FLAG_SHIFT;

        Self(bits)
    }

    pub fn promotion(from: Square, to: Square, promoted_piece: Piece, is_capture: bool) -> Self {
        let flag = if is_capture {
            match promoted_piece {
                Piece::Knight => MoveFlag::KnightPromotionCapture,
                Piece::Bishop => MoveFlag::BishopPromotionCapture,
                Piece::Rook => MoveFlag::RookPromotionCapture,
                Piece::Queen => MoveFlag::QueenPromotionCapture,
                _ => unreachable!(),
            }
        } else {
            match promoted_piece {
                Piece::Knight => MoveFlag::KnightPromotion,
                Piece::Bishop => MoveFlag::BishopPromotion,
                Piece::Rook => MoveFlag::RookPromotion,
                Piece::Queen => MoveFlag::QueenPromotion,
                _ => unreachable!(),
            }
        };
        let mut bits = 0;

        bits |= ((from.index() as u16) & FROM_MASK) << FROM_SHIFT;
        bits |= ((to.index() as u16) & TO_MASK) << TO_SHIFT;
        bits |= ((flag as u16) & FLAG_MASK) << FLAG_SHIFT;

        Self(bits)
    }

    pub const fn null_move() -> Self {
        Self(0)
    }

    #[inline(always)]
    pub const fn bits(&self) -> u16 {
        self.0
    }
}

impl Move {
    #[inline(always)]
    pub const fn from(&self) -> Square {
        let index = (self.0 >> FROM_SHIFT) & FROM_MASK;
        Square::from_index(index as u8)
    }

    #[inline(always)]
    pub const fn to(&self) -> Square {
        let index = (self.0 >> TO_SHIFT) & TO_MASK;
        Square::from_index(index as u8)
    }

    #[inline(always)]
    pub const fn flag(&self) -> MoveFlag {
        let flag = (self.0 >> FLAG_SHIFT) & FLAG_MASK;
        MoveFlag::from_flag(flag as u8)
    }

    // TODO: Make this better
    pub fn is_promotion(&self) -> bool {
        match self.flag() {
            MoveFlag::KnightPromotion
            | MoveFlag::BishopPromotion
            | MoveFlag::RookPromotion
            | MoveFlag::QueenPromotion
            | MoveFlag::KnightPromotionCapture
            | MoveFlag::BishopPromotionCapture
            | MoveFlag::RookPromotionCapture
            | MoveFlag::QueenPromotionCapture => true,
            _ => false,
        }
    }

    // TODO: Make this better
    pub fn is_capture(&self) -> bool {
        match self.flag() {
            MoveFlag::Capture
            | MoveFlag::EnPassant
            | MoveFlag::KnightPromotionCapture
            | MoveFlag::BishopPromotionCapture
            | MoveFlag::RookPromotionCapture
            | MoveFlag::QueenPromotionCapture => true,
            _ => false,
        }
    }

    #[inline(always)]
    pub fn is_double_pawn(&self) -> bool {
        self.flag() == MoveFlag::DoublePawn
    }

    #[inline(always)]
    pub fn is_en_passant(&self) -> bool {
        self.flag() == MoveFlag::EnPassant
    }

    // TODO: Make this better
    #[inline(always)]
    pub fn is_castling(&self) -> bool {
        match self.flag() {
            MoveFlag::KingCastle | MoveFlag::QueenCastle => true,
            _ => false,
        }
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

        let piece = match board.get_piece_type(from) {
            Some(colored_piece) => colored_piece.piece,
            None => return Err(PieceNotFound::new(from.to_string()).into()),
        };
        let is_capture = board.get_piece_type(to).is_some();

        let is_promotion = match input.len() {
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
            Some(en_passant) => en_passant.to_move == to && piece == Piece::Pawn,
            _ => false,
        };

        let mut is_double_pawn = piece == Piece::Pawn && (from.rank() == 1 || from.rank() == 6);
        if is_double_pawn {
            let diff = (i8::from(from) - i8::from(to)).abs();
            is_double_pawn = diff == 16;
        }

        let mov = if is_double_pawn {
            Move::double_pawn(from, to)
        } else if is_en_passant {
            Move::en_passant(from, to)
        } else if let Some(promoted) = is_promotion {
            Move::promotion(from, to, promoted, is_capture)
        } else if is_capture {
            Move::capture(from, to)
        } else if piece == Piece::King {
            match (from, to) {
                (E8, G8) | (E1, G1) => Move::castle(from, to, true),
                (E8, C8) | (E1, C1) => Move::castle(from, to, false),
                _ => Move::quiet(from, to),
            }
        } else {
            Move::quiet(from, to)
        };

        return Ok(mov);
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let from = self.from().to_string();
        let to = self.to().to_string();

        if self.is_promotion() {
            let promoted_piece = self.flag().get_promotion_piece();
            let colored_piece = ColoredPiece::new(promoted_piece, Color::Black);
            write!(f, "{}{}{}", from, to, colored_piece.to_fen())
        } else {
            write!(f, "{}{}", from, to)
        }
    }
}
