pub mod error;

use std::fmt::Display;
use std::ops::Not;
use std::str::FromStr;

use colored::Colorize;
use strum::{EnumCount, EnumIter, IntoEnumIterator};

use crate::{
    bitboard::{Bitboard, Square},
    move_generator::Move,
};

use self::error::{
    BoardError, ColoredPieceError, InvalidEnPassant, InvalidFenPiece, NotEnoughParts,
    PieceNotFound, WrongActiveColor, WrongCastlingAvailibility,
};

#[derive(Debug, Clone, Copy, EnumCount, EnumIter)]
pub enum Color {
    Black,
    White,
}

impl Color {
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

#[derive(Debug, Clone, Copy, EnumCount, EnumIter)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl Piece {
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

    pub fn get_squares_by_piece(&self, color: Color, piece: Piece) -> Vec<Square> {
        let mut squares = Vec::new();

        let mut pieces = *self.get_piece_board(color, piece);
        while pieces.bits != 0 {
            let index = pieces.bits.trailing_zeros() as usize;
            let from = Square::index(index as u8);
            pieces ^= from;

            squares.push(from);
        }

        squares
    }

    pub fn get_active_squares_by_piece(&self, piece: Piece) -> Vec<Square> {
        let color = self.active;
        self.get_squares_by_piece(color, piece)
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

    pub fn get_active_piece_board(&self, piece: Piece) -> &Bitboard {
        let color = self.active;
        self.get_piece_board(color, piece)
    }

    pub fn get_other_piece_board(&self, piece: Piece) -> &Bitboard {
        let color = !self.active;
        self.get_piece_board(color, piece)
    }

    pub fn get_color_occupied(&self, color: Color) -> &Bitboard {
        match color {
            Color::White => &self.white,
            Color::Black => &self.black,
        }
    }

    pub fn get_own_occupied(&self) -> &Bitboard {
        let color = self.active;
        self.get_color_occupied(color)
    }

    pub fn get_other_occpuied(&self) -> &Bitboard {
        let color = !self.active;
        self.get_color_occupied(color)
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

    pub fn toggle_active(&mut self, piece: Piece, square: Square) {
        let color = self.active;
        self.toggle(color, piece, square);
    }

    pub fn toggle_other(&mut self, piece: Piece, square: Square) {
        let color = !self.active;
        self.toggle(color, piece, square);
    }

    pub fn play(&mut self, color: Color, mov: &Move) -> Result<(), BoardError> {
        let from_bb: Bitboard = mov.from.into();
        let to_bb: Bitboard = mov.to.into();

        let bitboard = from_bb | to_bb;

        let color_index = color.index();
        let piece_index = mov.piece.index();
        self.bitboards[color_index][piece_index] ^= bitboard;

        if mov.attack {
            let color = !color;
            let piece = self.get_piece_type(color, mov.to).ok_or(PieceNotFound)?;
            let color_index = color.index();
            let piece_index = piece.index();
            self.bitboards[color_index][piece_index] ^= to_bb;
        }

        match color {
            Color::White => {
                self.white ^= bitboard;
                if mov.attack {
                    self.black ^= to_bb;
                }
            }
            Color::Black => {
                self.black ^= bitboard;
                if mov.attack {
                    self.white ^= to_bb;
                }
            }
        }

        if mov.attack {
            self.occupied ^= from_bb;
        } else {
            self.occupied ^= bitboard;
        }

        Ok(())
    }

    pub fn play_active(&mut self, mov: &Move) -> Result<(), BoardError> {
        let color = self.active;
        self.play(color, mov)
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

#[cfg(test)]
mod tests {
    use crate::board::{Board, Color, Piece};
    use std::str::FromStr;

    #[test]
    fn fen_starting_position() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let board = Board::from_str(fen).unwrap();

        let king_bb = board.get_piece_board(Color::White, Piece::King);
        assert_eq!(king_bb.bits, 0x10);
        let king_bb = board.get_piece_board(Color::Black, Piece::King);
        assert_eq!(king_bb.bits, 0x1000000000000000);

        let pawn_bb = board.get_piece_board(Color::White, Piece::Pawn);
        assert_eq!(pawn_bb.bits, 0xff00);
        let pawn_bb = board.get_piece_board(Color::Black, Piece::Pawn);
        assert_eq!(pawn_bb.bits, 0xff000000000000);

        let knight_bb = board.get_piece_board(Color::White, Piece::Knight);
        assert_eq!(knight_bb.bits, 0x42);
        let knight_bb = board.get_piece_board(Color::Black, Piece::Knight);
        assert_eq!(knight_bb.bits, 0x4200000000000000);

        let bishop_bb = board.get_piece_board(Color::White, Piece::Bishop);
        assert_eq!(bishop_bb.bits, 0x24);
        let bishop_bb = board.get_piece_board(Color::Black, Piece::Bishop);
        assert_eq!(bishop_bb.bits, 0x2400000000000000);

        let rook_bb = board.get_piece_board(Color::White, Piece::Rook);
        assert_eq!(rook_bb.bits, 0x81);
        let rook_bb = board.get_piece_board(Color::Black, Piece::Rook);
        assert_eq!(rook_bb.bits, 0x8100000000000000);

        let queen_bb = board.get_piece_board(Color::White, Piece::Queen);
        assert_eq!(queen_bb.bits, 0x8);
        let queen_bb = board.get_piece_board(Color::Black, Piece::Queen);
        assert_eq!(queen_bb.bits, 0x800000000000000);
    }

    #[test]
    fn fen_custom_1() {
        let fen = "rnbq1bnr/pppk1ppp/8/1B1pp3/3PP3/5P2/PPP3PP/RNBQK1NR b KQ - 2 4";
        let board = Board::from_str(fen).unwrap();

        let king_bb = board.get_piece_board(Color::White, Piece::King);
        assert_eq!(king_bb.bits, 0x10);
        let king_bb = board.get_piece_board(Color::Black, Piece::King);
        assert_eq!(king_bb.bits, 0x8000000000000);

        let pawn_bb = board.get_piece_board(Color::White, Piece::Pawn);
        assert_eq!(pawn_bb.bits, 0x1820c700);
        let pawn_bb = board.get_piece_board(Color::Black, Piece::Pawn);
        assert_eq!(pawn_bb.bits, 0xe7001800000000);

        let knight_bb = board.get_piece_board(Color::White, Piece::Knight);
        assert_eq!(knight_bb.bits, 0x42);
        let knight_bb = board.get_piece_board(Color::Black, Piece::Knight);
        assert_eq!(knight_bb.bits, 0x4200000000000000);

        let bishop_bb = board.get_piece_board(Color::White, Piece::Bishop);
        assert_eq!(bishop_bb.bits, 0x200000004);
        let bishop_bb = board.get_piece_board(Color::Black, Piece::Bishop);
        assert_eq!(bishop_bb.bits, 0x2400000000000000);

        let rook_bb = board.get_piece_board(Color::White, Piece::Rook);
        assert_eq!(rook_bb.bits, 0x81);
        let rook_bb = board.get_piece_board(Color::Black, Piece::Rook);
        assert_eq!(rook_bb.bits, 0x8100000000000000);

        let queen_bb = board.get_piece_board(Color::White, Piece::Queen);
        assert_eq!(queen_bb.bits, 0x8);
        let queen_bb = board.get_piece_board(Color::Black, Piece::Queen);
        assert_eq!(queen_bb.bits, 0x800000000000000);
    }
}
