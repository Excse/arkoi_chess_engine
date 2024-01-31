use std::{fmt::Display, str::FromStr};

use crate::{
    bitboard::{square::Square, squares::*},
    board::{Board, Color, ColoredPiece, Piece},
};

use super::error::{InvalidMoveFormat, MoveError, PieceNotFound};

#[derive(Debug, PartialEq, Eq)]
pub enum MoveKind {
    Normal,
    Attack,
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

#[derive(Debug, PartialEq, Eq)]
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
        let rank_difference = (self.to.rank as isize - self.from.rank as isize).abs();
        let should_en_passant = self.piece == Piece::Pawn && rank_difference == 2;
        if !should_en_passant {
            return None;
        }

        let to_capture = self.to;
        let to_move = Square::new(
            (self.from.rank + self.to.rank) / 2,
            (self.from.file + self.to.file) / 2,
        );

        Some(EnPassant::new(to_move, to_capture))
    }

    // En passant is an attack but its not directly attacking a piece
    pub fn is_direct_attack(&self) -> bool {
        match self.kind {
            MoveKind::Attack => true,
            MoveKind::Promotion(ref promotion) => promotion.is_attack,
            _ => false,
        }
    }

    pub fn get_attacking_square(&self) -> Option<Square> {
        match self.kind {
            MoveKind::Attack => Some(self.to),
            MoveKind::Promotion(ref promotion) if promotion.is_attack => Some(self.to),
            MoveKind::EnPassant(ref en_passant) => Some(en_passant.capture),
            _ => None,
        }
    }

    pub fn parse(input: String, color: Color, board: &Board) -> Result<Self, MoveError> {
        let promoted = input.len() == 5;

        if input.len() != 4 && !promoted {
            return Err(InvalidMoveFormat::new(input.clone()).into());
        }

        let from = Square::from_str(&input[0..2])?;
        let to = Square::from_str(&input[2..4])?;

        let piece = board
            .get_piece_type(board.active, from)
            .ok_or(PieceNotFound::new(from.to_string()))?;
        let attacked = board.get_piece_type(!board.active, to);

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

        // TODO: Add if its an en passant move or not and what piece to capture
        let mov = if promoted {
            let promoted_piece = promoted_piece.unwrap();
            PromotionMove::new(from, to, promoted_piece, attacked.is_some())
        } else if attacked.is_some() {
            AttackMove::new(piece, from, to)
        } else if piece == Piece::King {
            let (rook_from, rook_to) = match (color, from, to) {
                (Color::Black, E8, G8) => (H8, F8),
                (Color::Black, E8, C8) => (A8, D8),
                (Color::White, E1, G1) => (H1, F1),
                (Color::White, E1, C1) => (A1, D1),
                // TODO: THIS IS A BUG
                _ => (from, to),
            };
            CastleMove::new(color, from, to, rook_from, rook_to)
        } else {
            NormalMove::new(piece, from, to)
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

#[derive(Debug, PartialEq, Eq)]
pub struct EnPassantMove {
    pub capture: Square,
}

impl EnPassantMove {
    pub fn new(from: Square, to: Square, capture: Square) -> Move {
        Move::new(Piece::Pawn, from, to, MoveKind::EnPassant(Self { capture }))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct AttackMove;

impl AttackMove {
    pub fn new(piece: Piece, from: Square, to: Square) -> Move {
        Move::new(piece, from, to, MoveKind::Attack)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct PromotionMove {
    pub promotion: Piece,
    pub is_attack: bool,
}

impl PromotionMove {
    pub fn new(from: Square, to: Square, promotion: Piece, is_attack: bool) -> Move {
        Move::new(
            Piece::Pawn,
            from,
            to,
            MoveKind::Promotion(Self {
                promotion,
                is_attack,
            }),
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
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
