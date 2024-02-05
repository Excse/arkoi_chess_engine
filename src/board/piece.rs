use crate::{
    bitboard::square::Square,
    board::color::Color,
    lookup::pesto::{
        ENDGAME_BISHOP_TABLE, ENDGAME_KING_TABLE, ENDGAME_KNIGHT_TABLE, ENDGAME_PAWN_TABLE,
        ENDGAME_PIECE_VALUE, ENDGAME_QUEEN_TABLE, ENDGAME_ROOK_TABLE, GAMEPHASE_INCREMENT,
        MIDGAME_BISHOP_TABLE, MIDGAME_KING_TABLE, MIDGAME_KNIGHT_TABLE, MIDGAME_PAWN_TABLE,
        MIDGAME_PIECE_VALUE, MIDGAME_QUEEN_TABLE, MIDGAME_ROOK_TABLE,
    },
};

use super::error::{ColoredPieceError, InvalidFenPiece};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl Piece {
    pub const COUNT: usize = 6;

    pub const fn index(&self) -> usize {
        *self as usize
    }

    #[inline(always)]
    pub const fn get_midgame_value(&self) -> isize {
        MIDGAME_PIECE_VALUE[self.index()]
    }

    #[inline(always)]
    pub const fn get_endgame_value(&self) -> isize {
        ENDGAME_PIECE_VALUE[self.index()]
    }

    #[inline(always)]
    pub const fn get_gamephase_value(&self) -> isize {
        GAMEPHASE_INCREMENT[self.index()]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ColoredPiece {
    pub piece: Piece,
    pub color: Color,
}

impl ColoredPiece {
    pub const fn new(piece: Piece, color: Color) -> Self {
        Self { piece, color }
    }

    pub const fn get_midgame_square_value(&self, square: Square) -> isize {
        let index = match self.color {
            Color::White => square.index,
            Color::Black => 63 - square.index,
        };
        match self.piece {
            Piece::Pawn => MIDGAME_PAWN_TABLE[index],
            Piece::Knight => MIDGAME_KNIGHT_TABLE[index],
            Piece::Bishop => MIDGAME_BISHOP_TABLE[index],
            Piece::Rook => MIDGAME_ROOK_TABLE[index],
            Piece::Queen => MIDGAME_QUEEN_TABLE[index],
            Piece::King => MIDGAME_KING_TABLE[index],
        }
    }

    pub const fn get_endgame_square_value(&self, square: Square) -> isize {
        let index = match self.color {
            Color::White => square.index,
            Color::Black => 63 - square.index,
        };
        match self.piece {
            Piece::Pawn => ENDGAME_PAWN_TABLE[index],
            Piece::Knight => ENDGAME_KNIGHT_TABLE[index],
            Piece::Bishop => ENDGAME_BISHOP_TABLE[index],
            Piece::Rook => ENDGAME_ROOK_TABLE[index],
            Piece::Queen => ENDGAME_QUEEN_TABLE[index],
            Piece::King => ENDGAME_KING_TABLE[index],
        }
    }

    pub fn from_fen(piece: char) -> Result<Self, ColoredPieceError> {
        match piece {
            'P' => Ok(Self::new(Piece::Pawn, Color::White)),
            'p' => Ok(Self::new(Piece::Pawn, Color::Black)),
            'N' => Ok(Self::new(Piece::Knight, Color::White)),
            'n' => Ok(Self::new(Piece::Knight, Color::Black)),
            'B' => Ok(Self::new(Piece::Bishop, Color::White)),
            'b' => Ok(Self::new(Piece::Bishop, Color::Black)),
            'R' => Ok(Self::new(Piece::Rook, Color::White)),
            'r' => Ok(Self::new(Piece::Rook, Color::Black)),
            'Q' => Ok(Self::new(Piece::Queen, Color::White)),
            'q' => Ok(Self::new(Piece::Queen, Color::Black)),
            'K' => Ok(Self::new(Piece::King, Color::White)),
            'k' => Ok(Self::new(Piece::King, Color::Black)),
            _ => Err(InvalidFenPiece::new(piece).into()),
        }
    }

    pub const fn to_fen(&self) -> char {
        match (self.color, self.piece) {
            (Color::White, Piece::Pawn) => 'P',
            (Color::White, Piece::Knight) => 'N',
            (Color::White, Piece::Bishop) => 'B',
            (Color::White, Piece::Rook) => 'R',
            (Color::White, Piece::Queen) => 'Q',
            (Color::White, Piece::King) => 'K',

            (Color::Black, Piece::Pawn) => 'p',
            (Color::Black, Piece::Knight) => 'n',
            (Color::Black, Piece::Bishop) => 'b',
            (Color::Black, Piece::Rook) => 'r',
            (Color::Black, Piece::Queen) => 'q',
            (Color::Black, Piece::King) => 'k',
        }
    }
}
