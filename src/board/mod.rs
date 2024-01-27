pub mod error;
mod tests;

use std::fmt::Display;
use std::ops::Not;
use std::str::FromStr;

use colored::Colorize;
use strum::{EnumIter, IntoEnumIterator};

use crate::bitboard::{Bitboard, Square};
use crate::move_generator::Move;

use self::error::{
    BoardError, ColoredPieceError, InvalidEnPassant, InvalidFenPiece, MultipleKings,
    NotEnoughParts, PieceNotFound, WrongActiveColor, WrongCastlingAvailibility,
};

#[derive(Debug, Clone, Copy, EnumIter, PartialEq, Eq)]
pub enum Color {
    Black,
    White,
}

impl Color {
    pub const COUNT: usize = 2;

    pub fn index(&self) -> usize {
        *self as usize
    }

    pub fn at(index: usize) -> Option<Self> {
        Color::iter().nth(index)
    }
}

impl Not for Color {
    type Output = Color;

    fn not(self) -> Self::Output {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}

#[derive(Debug, Clone, Copy, EnumIter, PartialEq, Eq, PartialOrd, Ord)]
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

    fn index(&self) -> usize {
        *self as usize
    }

    fn at(index: usize) -> Option<Self> {
        Piece::iter().nth(index)
    }
}

pub struct ColoredPiece {
    pub piece: Piece,
    pub color: Color,
}

impl ColoredPiece {
    pub fn new(piece: Piece, color: Color) -> Self {
        Self { piece, color }
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

    pub fn to_fen(&self) -> char {
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

#[derive(Debug, Clone, Copy)]
pub struct Board {
    pub bitboards: [[Bitboard; Piece::COUNT]; Color::COUNT],
    pub white: Bitboard,
    pub black: Bitboard,
    pub occupied: Bitboard,
    pub active: Color,
    pub black_kingside: bool,
    pub white_kingside: bool,
    pub black_queenside: bool,
    pub white_queenside: bool,
    pub en_passant: Option<Square>,
    pub halfemoves: u16,
    pub fullmoves: u16,
}

impl Default for Board {
    fn default() -> Self {
        Board::from_str(Self::STARTPOS_FEN).unwrap()
    }
}

impl Board {
    pub const STARTPOS_FEN: &'static str =
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    pub const MAX_RANK: usize = 7;
    pub const MAX_FILE: usize = 7;
    pub const MIN_RANK: usize = 0;
    pub const MIN_FILE: usize = 0;
    pub const SIZE: usize = 64;

    pub fn empty() -> Board {
        Board {
            bitboards: [[Bitboard::default(); Piece::COUNT]; Color::COUNT],
            occupied: Bitboard::default(),
            white: Bitboard::default(),
            black: Bitboard::default(),
            active: Color::White,
            black_kingside: false,
            white_kingside: false,
            black_queenside: false,
            white_queenside: false,
            en_passant: None,
            halfemoves: 0,
            fullmoves: 0,
        }
    }

    pub fn swap_active(&mut self) {
        self.active = !self.active
    }

    pub fn get_king_square(&self, color: Color) -> Result<Option<Square>, BoardError> {
        let kings = self.get_squares_by_piece(color, Piece::King);

        match kings.len() {
            0 => Ok(None),
            1 => Ok(Some(kings[0])),
            _ => Err(MultipleKings::new(kings.len()).into()),
        }
    }

    pub fn get_squares_by_piece(&self, color: Color, piece: Piece) -> Vec<Square> {
        let mut squares = Vec::new();

        let mut pieces = *self.get_piece_board(color, piece);

        while pieces.bits != 0 {
            let index = pieces.bits.trailing_zeros() as usize;
            let from = Square::index(index);
            pieces ^= from;

            squares.push(from);
        }

        squares
    }

    pub fn get_piece_type(&self, color: Color, square: Square) -> Option<Piece> {
        let bitboards = &self.bitboards[color.index()];
        for (index, &piece_bb) in bitboards.iter().enumerate() {
            let contains_bb = piece_bb & square;
            if contains_bb.bits != 0 {
                let piece = Piece::at(index)?;
                return Some(piece);
            }
        }

        None
    }

    pub fn get_colored_piece_type(&self, square: Square) -> Option<ColoredPiece> {
        for (color_index, &color_bb) in self.bitboards.iter().enumerate() {
            for (piece_index, &piece_bb) in color_bb.iter().enumerate() {
                let contains_bb = piece_bb & square;
                if contains_bb.bits != 0 {
                    let piece = Piece::at(piece_index)?;
                    let color = Color::at(color_index)?;
                    let piece = ColoredPiece::new(piece, color);
                    return Some(piece);
                }
            }
        }

        None
    }

    pub fn get_piece_board(&self, color: Color, piece: Piece) -> &Bitboard {
        let index = color.index();
        let bitboards = &self.bitboards[index];

        let index = piece.index();
        &bitboards[index]
    }

    pub fn get_occupied(&self, color: Color) -> &Bitboard {
        match color {
            Color::White => &self.white,
            Color::Black => &self.black,
        }
    }

    #[inline]
    pub fn get_all_occupied(&self) -> &Bitboard {
        &self.occupied
    }

    pub fn toggle(&mut self, color: Color, piece: Piece, square: Square) {
        let color_index = color.index();
        let piece_index = piece.index();
        self.bitboards[color_index][piece_index] ^= square;

        match color {
            Color::White => self.white ^= square,
            Color::Black => self.black ^= square,
        }

        self.occupied ^= square;
    }

    pub fn play(&mut self, color: Color, mov: &Move) -> Result<(), BoardError> {
        if *mov == Move::OO_KING_WHITE {
            self.play(color, &Move::OO_ROOK_WHITE)?;
        } else if *mov == Move::OO_KING_BLACK {
            self.play(color, &Move::OO_ROOK_BLACK)?;
        } else if *mov == Move::OOO_KING_WHITE {
            self.play(color, &Move::OOO_ROOK_WHITE)?;
        } else if *mov == Move::OOO_KING_BLACK {
            self.play(color, &Move::OOO_ROOK_BLACK)?;
        }

        if mov.promoted {
            self.toggle(color, Piece::Pawn, mov.from);
            self.toggle(!color, mov.piece, mov.to);
        } else {
            self.toggle(color, mov.piece, mov.from);
            self.toggle(color, mov.piece, mov.to);
        }

        if mov.attack {
            let color = !color;
            let piece = self.get_piece_type(color, mov.to).ok_or(PieceNotFound)?;
            self.toggle(color, piece, mov.to);
        }

        Ok(())
    }

    pub fn unplay(&mut self, color: Color, mov: &Move) -> Result<(), BoardError> {
        if *mov == Move::OO_KING_WHITE {
            self.play(color, &Move::OO_ROOK_REVERSE_WHITE)?;
        } else if *mov == Move::OO_KING_BLACK {
            self.play(color, &Move::OO_ROOK_REVERSE_BLACK)?;
        } else if *mov == Move::OOO_KING_WHITE {
            self.play(color, &Move::OOO_ROOK_REVERSE_WHITE)?;
        } else if *mov == Move::OOO_KING_BLACK {
            self.play(color, &Move::OOO_ROOK_REVERSE_BLACK)?;
        }

        if mov.promoted {
            self.toggle(color, Piece::Pawn, mov.from);
            self.toggle(!color, mov.piece, mov.to);
        } else {
            self.toggle(color, mov.piece, mov.from);
            self.toggle(color, mov.piece, mov.to);
        }

        if mov.attack {
            let color = !color;
            let piece = self.get_piece_type(color, mov.to).ok_or(PieceNotFound)?;
            self.toggle(color, piece, mov.to);
        }

        Ok(())
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for rank in (0..8).rev() {
            for size in 0..3 {
                if size == 1 {
                    write!(f, "{} ", rank + 1)?;
                } else {
                    write!(f, "  ")?;
                }

                for file in 0..8 {
                    let index = (8 * rank) + file;
                    let color = (index + rank) % 2;

                    let square = Square::new(rank, file);
                    let piece = self.get_colored_piece_type(square);
                    let piece = match (size, piece) {
                        (1, Some(piece)) => piece.to_fen(),
                        (_, _) => ' ',
                    };
                    let piece = format!("   {}   ", piece);

                    if color == 0 {
                        write!(f, "{}", piece.white().on_black())?;
                    } else {
                        write!(f, "{}", piece.black().on_white())?;
                    }
                }
                writeln!(f)?;
            }
        }

        writeln!(f, "     a      b      c      d      e      f      g      h")
    }
}

impl FromStr for Board {
    type Err = BoardError;

    fn from_str(fen: &str) -> Result<Self, BoardError> {
        let fen_parts: Vec<&str> = fen.split(" ").collect();
        if fen_parts.len() != 6 {
            return Err(NotEnoughParts.into());
        }

        let mut board = Board::empty();

        let ranks = fen_parts[0].split("/");
        for (rank_index, rank) in ranks.enumerate() {
            let mut file_index: u8 = 0;
            for piece in rank.chars() {
                if piece.is_digit(10) {
                    let digit = piece.to_digit(10).unwrap();
                    file_index += digit as u8;
                    continue;
                }

                let rank_index = 7 - rank_index as u8;

                let ColoredPiece { piece, color } = ColoredPiece::from_fen(piece)?;
                let square = Square::new(rank_index, file_index);
                board.toggle(color, piece, square);

                file_index += 1;
            }
        }

        let active_color = fen_parts[1];
        match active_color {
            "w" => board.active = Color::White,
            "b" => board.active = Color::Black,
            _ => return Err(WrongActiveColor::new(active_color).into()),
        }

        let castling_availibility = fen_parts[2];
        for availibility in castling_availibility.chars() {
            if availibility == '-' {
                break;
            }

            let piece = ColoredPiece::from_fen(availibility)?;
            match (piece.color, piece.piece) {
                (Color::Black, Piece::Queen) => board.black_queenside = true,
                (Color::White, Piece::Queen) => board.white_queenside = true,
                (Color::Black, Piece::King) => board.black_kingside = true,
                (Color::White, Piece::King) => board.white_kingside = true,
                _ => return Err(WrongCastlingAvailibility::new(availibility).into()),
            }
        }

        let en_passant = fen_parts[3];
        if en_passant != "-" {
            let file = en_passant
                .chars()
                .nth(0)
                .ok_or(InvalidEnPassant::new(en_passant))?;
            let file = file as u8 - b'a';

            let rank = en_passant
                .chars()
                .nth(1)
                .ok_or(InvalidEnPassant::new(en_passant))?;
            let rank = rank.to_digit(10).ok_or(InvalidEnPassant::new(en_passant))?;
            if rank < 1 || rank > 8 {
                return Err(InvalidEnPassant::new(en_passant).into());
            }

            let rank = rank as u8 - 1;
            let square = Square::new(rank, file);
            board.en_passant = Some(square);
        }

        let halfemoves = fen_parts[4].parse::<u16>()?;
        board.halfemoves = halfemoves;

        let fullmoves = fen_parts[5].parse::<u16>()?;
        board.fullmoves = fullmoves;

        Ok(board)
    }
}
