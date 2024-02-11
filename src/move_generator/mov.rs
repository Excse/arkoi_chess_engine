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

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum MoveKind {
    Normal,
    Attack(AttackMove),
    EnPassant(EnPassantMove),
    Promotion(PromotionMove),
    Castle(CastleMove),
}

#[derive(Debug, Clone, Copy)]
pub struct EnPassant {
    pub to_move: Square,
    pub to_capture: Square,
}

impl EnPassant {
    pub fn new(to_move: Square, to_capture: Square) -> Self {
        Self {
            to_move,
            to_capture,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct Move {
    pub piece: Piece,
    pub from: Square,
    pub to: Square,
    pub kind: MoveKind,
}

impl Move {
    pub const fn new(piece: Piece, from: Square, to: Square, kind: MoveKind) -> Self {
        Self {
            piece,
            from,
            to,
            kind,
        }
    }

    pub fn is_en_passant(&self) -> Option<EnPassant> {
        let index_difference = (self.to.index as isize - self.from.index as isize).abs();
        let should_en_passant = self.piece == Piece::Pawn && index_difference == 16;
        if !should_en_passant {
            return None;
        }

        let to_capture = self.to;
        let to_move = Square::index((self.from.index + self.to.index) / 2);

        Some(EnPassant::new(to_move, to_capture))
    }

    pub fn is_direct_attack(&self) -> bool {
        match self.kind {
            MoveKind::Attack(_) => true,
            MoveKind::Promotion(ref promotion) => promotion.attacked.is_some(),
            _ => false,
        }
    }

    pub fn is_attack(&self) -> bool {
        match self.kind {
            MoveKind::Attack(_) => true,
            MoveKind::Promotion(ref promotion) => promotion.attacked.is_some(),
            MoveKind::EnPassant(_) => true,
            _ => false,
        }
    }

    pub fn get_attacking_square(&self) -> Option<Square> {
        match self.kind {
            MoveKind::Attack(_) => Some(self.to),
            MoveKind::Promotion(ref promotion) if promotion.attacked.is_some() => Some(self.to),
            MoveKind::EnPassant(ref en_passant) => Some(en_passant.capture),
            _ => None,
        }
    }

    pub fn parse(input: String, board: &Board) -> Result<Self, MoveError> {
        let promoted = input.len() == 5;

        if input.len() != 4 && !promoted {
            return Err(InvalidMoveFormat::new(input.clone()).into());
        }

        let from = Square::from_str(&input[0..2])?;
        let to = Square::from_str(&input[2..4])?;

        let colored_piece = board
            .get_piece_type(from)
            .ok_or(PieceNotFound::new(from.to_string()))?;
        let attacked = match board.get_piece_type(to) {
            Some(colored_piece) => Some(colored_piece.piece),
            _ => None,
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

        let mut is_en_passant = false;
        if let Some(en_passant) = board.en_passant {
            is_en_passant = en_passant.to_move == to;
        }

        let mov = if colored_piece.piece == Piece::Pawn && is_en_passant {
            let mut capture_index = to.index;
            if colored_piece.color == Color::White {
                capture_index -= 8;
            } else {
                capture_index += 8;
            }

            let capture = Square::index(capture_index);
            EnPassantMove::new(from, to, capture)
        } else if promoted {
            let promoted_piece = promoted_piece.unwrap();
            PromotionMove::new(from, to, promoted_piece, attacked)
        } else if let Some(attacked) = attacked {
            AttackMove::new(colored_piece.piece, from, attacked, to)
        } else if colored_piece.piece == Piece::King {
            match (colored_piece.color, from, to) {
                (Color::Black, E8, G8) => CastleMove::KING_BLACK,
                (Color::Black, E8, C8) => CastleMove::QUEEN_BLACK,
                (Color::White, E1, G1) => CastleMove::KING_WHITE,
                (Color::White, E1, C1) => CastleMove::QUEEN_WHITE,
                _ => NormalMove::new(Piece::King, from, to),
            }
        } else {
            NormalMove::new(colored_piece.piece, from, to)
        };

        return Ok(mov);
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let from = self.from.to_string();
        let to = self.to.to_string();

        match self.kind {
            MoveKind::Promotion(ref promotion) => {
                let colored_piece = ColoredPiece::new(promotion.promotion, Color::Black);
                write!(f, "{}{}{}", from, to, colored_piece.to_fen())
            }
            _ => {
                write!(f, "{}{}", from, to)
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct NormalMove;

impl NormalMove {
    pub fn new(piece: Piece, from: Square, to: Square) -> Move {
        Move::new(piece, from, to, MoveKind::Normal)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct EnPassantMove {
    pub capture: Square,
}

impl EnPassantMove {
    pub fn new(from: Square, to: Square, capture: Square) -> Move {
        Move::new(Piece::Pawn, from, to, MoveKind::EnPassant(Self { capture }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct AttackMove {
    pub attacked: Piece,
}

impl AttackMove {
    pub fn new(piece: Piece, from: Square, attacked: Piece, to: Square) -> Move {
        Move::new(piece, from, to, MoveKind::Attack(Self { attacked }))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct PromotionMove {
    pub promotion: Piece,
    pub attacked: Option<Piece>,
}

impl PromotionMove {
    pub fn new(from: Square, to: Square, promotion: Piece, attacked: Option<Piece>) -> Move {
        Move::new(
            Piece::Pawn,
            from,
            to,
            MoveKind::Promotion(Self {
                promotion,
                attacked,
            }),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct CastleMove {
    pub color: Color,
    pub rook_from: Square,
    pub rook_to: Square,
}

impl CastleMove {
    pub const QUEEN_WHITE: Move = Self::new(Color::White, E1, C1, A1, D1);
    pub const QUEEN_BLACK: Move = Self::new(Color::Black, E8, C8, A8, D8);

    pub const KING_WHITE: Move = Self::new(Color::White, E1, G1, H1, F1);
    pub const KING_BLACK: Move = Self::new(Color::Black, E8, G8, H8, F8);

    pub const fn new(
        color: Color,
        from: Square,
        to: Square,
        rook_from: Square,
        rook_to: Square,
    ) -> Move {
        Move::new(
            Piece::King,
            from,
            to,
            MoveKind::Castle(Self {
                color,
                rook_from,
                rook_to,
            }),
        )
    }
}
