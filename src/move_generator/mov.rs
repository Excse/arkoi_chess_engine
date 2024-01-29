use std::{fmt::Display, str::FromStr};

use crate::{
    bitboard::Square,
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

#[derive(Debug, PartialEq, Eq)]
pub struct Move {
    pub piece: Piece,
    pub from: Square,
    pub to: Square,
    pub kind: MoveKind,
}

impl Move {
    pub fn new(piece: Piece, from: Square, to: Square, kind: MoveKind) -> Self {
        Self {
            piece,
            from,
            to,
            kind,
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
            let (rook_from, rook_to) = match (color, from.index, to.index) {
                (Color::Black, 60, 62) => (63, 61),
                (Color::Black, 60, 58) => (56, 59),
                (Color::White, 4, 6) => (7, 5),
                (Color::White, 4, 2) => (0, 3),
                _ => (0, 0),
            };
            CastleMove::new(
                color,
                from,
                to,
                Square::index(rook_from),
                Square::index(rook_to),
            )
        } else {
            NormalMove::new(piece, from, to)
        };

        return Ok(mov);
    }
}

impl Move {
    // En passant is an attack but its not directly attacking a piece
    pub fn is_direct_attack(&self) -> bool {
        match self.kind {
            MoveKind::Attack => true,
            MoveKind::Promotion(ref promotion) => promotion.is_attack,
            _ => false,
        }
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
    // #[rustfmt::skip]
    // pub const OO_WHITE: Move = Self::new(Color::White, Square::index(4), Square::index(6), Square::index(7), Square::index(3));
    // #[rustfmt::skip]
    // pub const OO_BLACK: Move = Self::new(Color::Black, Square::index(60), Square::index(62), Square::index(63), Square::index(61));

    // #[rustfmt::skip]
    // pub const OOO_WHITE: Move = Self::new(Color::White, Square::index(4), Square::index(2), Square::index(0), Square::index(3));
    // #[rustfmt::skip]
    // pub const OOO_BLACK: Move = Self::new(Color::Black, Square::index(60), Square::index(58), Square::index(56), Square::index(59));

    pub fn new(color: Color, from: Square, to: Square, rook_from: Square, rook_to: Square) -> Move {
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
