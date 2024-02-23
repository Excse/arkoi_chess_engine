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

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
#[rustfmt::skip]
pub enum MoveFlag {
    Quiet                   = 0b0000,
    DoublePawn              = 0b0001,
    KingCastle              = 0b0010,
    QueenCastle             = 0b0011,
    Capture                 = 0b0100,
    EnPassant               = 0b0101,
    KnightPromotion         = 0b1000,
    BishopPromotion         = 0b1001,
    RookPromotion           = 0b1010,
    QueenPromotion          = 0b1011,
    KnightPromotionCapture  = 0b1100,
    BishopPromotionCapture  = 0b1101,
    RookPromotionCapture    = 0b1110,
    QueenPromotionCapture   = 0b1111,
}

impl MoveFlag {
    pub fn from_flag(flag: u8) -> Self {
        match flag {
            0b0000 => MoveFlag::Quiet,
            0b0001 => MoveFlag::DoublePawn,
            0b0010 => MoveFlag::KingCastle,
            0b0011 => MoveFlag::QueenCastle,
            0b0100 => MoveFlag::Capture,
            0b0101 => MoveFlag::EnPassant,
            0b1000 => MoveFlag::KnightPromotion,
            0b1001 => MoveFlag::BishopPromotion,
            0b1010 => MoveFlag::RookPromotion,
            0b1011 => MoveFlag::QueenPromotion,
            0b1100 => MoveFlag::KnightPromotionCapture,
            0b1101 => MoveFlag::BishopPromotionCapture,
            0b1110 => MoveFlag::RookPromotionCapture,
            0b1111 => MoveFlag::QueenPromotionCapture,
            _ => unreachable!(),
        }
    }

    pub fn get_promotion_piece(&self) -> Piece {
        match self {
            MoveFlag::KnightPromotion | MoveFlag::KnightPromotionCapture => Piece::Knight,
            MoveFlag::BishopPromotion | MoveFlag::BishopPromotionCapture => Piece::Bishop,
            MoveFlag::RookPromotion | MoveFlag::RookPromotionCapture => Piece::Rook,
            MoveFlag::QueenPromotion | MoveFlag::QueenPromotionCapture => Piece::Queen,
            _ => Piece::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct Move(u16);

impl Move {
    pub fn new(
        piece: Piece,
        from: Square,
        to: Square,
        promoted_piece: Piece,
        is_capture: bool,
        is_castling: bool,
        is_en_passant: bool,
    ) -> Self {
        let mut bits = 0;

        let is_double_pawn = if piece == Piece::Pawn {
            let square_difference = (isize::from(to) - isize::from(from)).abs();
            square_difference == 16
        } else {
            false
        };

        let flags = if is_capture && promoted_piece != Piece::None {
            match promoted_piece {
                Piece::Knight => MoveFlag::KnightPromotionCapture,
                Piece::Bishop => MoveFlag::BishopPromotionCapture,
                Piece::Rook => MoveFlag::RookPromotionCapture,
                Piece::Queen => MoveFlag::QueenPromotionCapture,
                _ => unreachable!(),
            }
        } else if promoted_piece != Piece::None {
            match promoted_piece {
                Piece::Knight => MoveFlag::KnightPromotion,
                Piece::Bishop => MoveFlag::BishopPromotion,
                Piece::Rook => MoveFlag::RookPromotion,
                Piece::Queen => MoveFlag::QueenPromotion,
                _ => unreachable!(),
            }
        } else if is_en_passant {
            MoveFlag::EnPassant
        } else if is_castling {
            match to {
                G1 | G8 => MoveFlag::KingCastle,
                C1 | C8 => MoveFlag::QueenCastle,
                _ => unreachable!(),
            }
        } else if is_capture {
            MoveFlag::Capture
        } else if piece == Piece::Pawn && is_double_pawn {
            MoveFlag::DoublePawn
        } else {
            MoveFlag::Quiet
        };

        bits |= ((from.index() as u16) & FROM_MASK) << FROM_SHIFT;
        bits |= ((to.index() as u16) & TO_MASK) << TO_SHIFT;
        bits |= ((flags as u16) & FLAG_MASK) << FLAG_SHIFT;

        Self(bits)
    }

    pub fn quiet(piece: Piece, from: Square, to: Square) -> Self {
        Self::new(piece, from, to, Piece::None, false, false, false)
    }

    pub fn capture(piece: Piece, from: Square, to: Square) -> Self {
        Self::new(piece, from, to, Piece::None, true, false, false)
    }

    pub fn en_passant(from: Square, to: Square) -> Self {
        Self::new(Piece::Pawn, from, to, Piece::None, false, false, true)
    }

    pub fn castle(from: Square, to: Square) -> Self {
        Self::new(Piece::King, from, to, Piece::None, false, true, false)
    }

    pub fn promotion(from: Square, to: Square, promoted_piece: Piece, is_capture: bool) -> Self {
        Self::new(
            Piece::Pawn,
            from,
            to,
            promoted_piece,
            is_capture,
            false,
            false,
        )
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
    pub fn is_double_pawn(&self) -> bool {
        let flag = self.flag();
        flag == MoveFlag::DoublePawn as u8
    }

    #[inline(always)]
    pub const fn is_castling(&self) -> bool {
        let flag = self.flag();
        flag == MoveFlag::KingCastle as u8 || flag == MoveFlag::QueenCastle as u8
    }

    #[inline(always)]
    pub const fn is_en_passant(&self) -> bool {
        let flag = self.flag();
        flag == MoveFlag::EnPassant as u8
    }

    #[inline(always)]
    pub const fn is_quiet(&self) -> bool {
        !self.is_capture()
    }

    #[inline(always)]
    pub const fn is_capture(&self) -> bool {
        let flag = (self.flag() as u16 & CAPTURE_FLAG_MASK) >> CAPTURE_FLAG_SHIFT;
        flag != 0
    }

    #[inline(always)]
    pub const fn is_promotion(&self) -> bool {
        let flag = (self.flag() as u16 & PROMOTION_FLAG_MASK) >> PROMOTION_FLAG_SHIFT;
        flag != 0
    }

    #[inline(always)]
    pub const fn flag(&self) -> u8 {
        let flag = (self.0 >> FLAG_SHIFT) & FLAG_MASK;
        flag as u8
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
        let is_capture = board.get_piece_type(to).is_some();

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
            Move::en_passant(from, to)
        } else if let Some(promoted) = promoted_piece {
            Move::promotion(from, to, promoted, is_capture)
        } else if is_capture {
            Move::capture(colored_piece.piece, from, to)
        } else if colored_piece.piece == Piece::King {
            match (colored_piece.color, from, to) {
                (Color::Black, E8, G8)
                | (Color::Black, E8, C8)
                | (Color::White, E1, G1)
                | (Color::White, E1, C1) => Move::castle(from, to),
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
            let flag = MoveFlag::from_flag(self.flag());
            let promoted_piece = flag.get_promotion_piece();
            let colored_piece = ColoredPiece::new(promoted_piece, Color::Black);
            write!(f, "{}{}{}", from, to, colored_piece.to_fen())
        } else {
            write!(f, "{}{}", from, to)
        }
    }
}
