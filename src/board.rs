use std::{num::ParseIntError, str::FromStr};

use thiserror::Error;

use crate::{bitboard::Bitboard, move_generator::Move};

pub type Result<T> = std::result::Result<T, BoardError>;

#[derive(Debug, Error)]
pub enum BoardError {
    #[error("there are not enough parts for this FEN")]
    NotEnoughParts,
    #[error("the active color '{0}' is not valid. You can only use 'w' or 'b'")]
    WrongActiveColor(String),
    #[error("the castling availibilty '{0}' is not valid. You can only use 'Q', 'K', 'q' or 'k'")]
    WrongCastlingAvailibility(char),
    #[error("the given piece '{0}' is not valid. You can only use 'k', 'q', 'r', 'p', 'b', 'n' in upper or lower case")]
    InvalidFenPiece(char),
    #[error("the given en passant square '{0}' is not valid")]
    InvalidEnPassant(String),
    #[error("{0}")]
    ParseIntError(ParseIntError),
}

impl From<ParseIntError> for BoardError {
    fn from(value: ParseIntError) -> Self {
        BoardError::ParseIntError(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub const AMOUNT: usize = 2;

    pub fn index(&self) -> usize {
        match self {
            Color::White => 1,
            Color::Black => 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl Piece {
    pub const AMOUNT: usize = 6;

    fn index(&self) -> usize {
        match self {
            Piece::Pawn => 0,
            Piece::Knight => 1,
            Piece::Bishop => 2,
            Piece::Rook => 3,
            Piece::Queen => 4,
            Piece::King => 5,
        }
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

    pub fn from_fen(piece: char) -> Result<Self> {
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
            _ => Err(BoardError::InvalidFenPiece(piece)),
        }
    }
}

#[derive(Debug)]
pub struct Board {
    pub bitboards: [[Bitboard; Piece::AMOUNT]; Color::AMOUNT],
    pub white: Bitboard,
    pub black: Bitboard,
    pub occupied: Bitboard,
    pub active: Color,
    pub black_kingside: bool,
    pub white_kingside: bool,
    pub black_queenside: bool,
    pub white_queenside: bool,
    pub en_passant: Option<Bitboard>,
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

    pub fn empty() -> Board {
        Board {
            bitboards: [[Bitboard::default(); Piece::AMOUNT]; Color::AMOUNT],
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
        match self.active {
            Color::White => self.active = Color::Black,
            Color::Black => self.active = Color::White,
        }
    }

    pub fn get_piece_board(&self, color: Color, piece: Piece) -> &Bitboard {
        let index = color.index();
        let bitboards = &self.bitboards[index];

        let index = piece.index();
        &bitboards[index]
    }

    pub fn get_active_piece_board(&self, piece: Piece) -> &Bitboard {
        let color = self.active;
        self.get_piece_board(color, piece)
    }

    pub fn get_color(&self, color: Color) -> &Bitboard {
        match color {
            Color::White => &self.white,
            Color::Black => &self.black,
        }
    }

    pub fn get_active(&self) -> &Bitboard {
        let color = self.active;
        self.get_color(color)
    }

    pub fn get_unactive(&self) -> &Bitboard {
        match self.active {
            Color::Black => self.get_color(Color::White),
            Color::White => self.get_color(Color::Black),
        }
    }

    #[inline]
    pub fn get_occupied(&self) -> &Bitboard {
        &self.occupied
    }

    pub fn play(&mut self, color: Color, mov: &Move) {
        println!("{:?} {}", mov, mov);
        let bitboard = mov.from | mov.to;

        let index = color.index();
        let bitboards = &mut self.bitboards[index];

        let index = mov.piece.index();
        bitboards[index] ^= bitboard;

        match color {
            Color::White => self.white ^= bitboard,
            Color::Black => self.black ^= bitboard,
        }

        self.occupied ^= bitboard;
    }

    pub fn play_active(&mut self, mov: &Move) {
        let color = self.active;
        self.play(color, mov)
    }
}

impl FromStr for Board {
    type Err = BoardError;

    fn from_str(fen: &str) -> Result<Self> {
        let fen_parts: Vec<&str> = fen.split(" ").collect();
        if fen_parts.len() != 6 {
            return Err(BoardError::NotEnoughParts);
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

                let square = Bitboard::square(rank_index, file_index);
                let ColoredPiece { piece, color } = ColoredPiece::from_fen(piece)?;
                let mov = Move::toggle(piece, square);
                board.play(color, &mov);

                file_index += 1;
            }
        }

        let active_color = fen_parts[1];
        match active_color {
            "w" => board.active = Color::White,
            "b" => board.active = Color::Black,
            _ => return Err(BoardError::WrongActiveColor(active_color.to_string())),
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
                _ => return Err(BoardError::WrongCastlingAvailibility(availibility)),
            }
        }

        let en_passant = fen_parts[3];
        if en_passant != "-" {
            let file = en_passant
                .chars()
                .nth(0)
                .ok_or(BoardError::InvalidEnPassant(en_passant.to_string()))?;
            let file = file as u8 - b'a';

            let rank = en_passant
                .chars()
                .nth(1)
                .ok_or(BoardError::InvalidEnPassant(en_passant.to_string()))?;
            let rank = rank
                .to_digit(10)
                .ok_or(BoardError::InvalidEnPassant(en_passant.to_string()))?;
            if rank < 1 || rank > 8 {
                return Err(BoardError::InvalidEnPassant(en_passant.to_string()));
            }

            let rank = rank as u8 - 1;
            let square = Bitboard::square(rank, file);
            board.en_passant = square.into();
        }

        let halfemoves = fen_parts[4].parse::<u16>()?;
        board.halfemoves = halfemoves;

        let fullmoves = fen_parts[5].parse::<u16>()?;
        board.fullmoves = fullmoves;

        Ok(board)
    }
}

#[cfg(test)]
mod tests {
    use crate::board::Board;
    use std::str::FromStr;

    #[test]
    fn fen_starting_position() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let board = Board::from_str(fen);
        println!("{:?}", board);
    }
}
