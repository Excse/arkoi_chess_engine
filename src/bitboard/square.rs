use std::{fmt::Display, str::FromStr};

use crate::{
    board::{color::Color, piece::Piece, Board},
    lookup::{pesto::*, tables, utils::Direction},
    generation::mov::Move,
};

use super::{
    error::{InvalidSquareFormat, SquareError},
    Bitboard,
};

#[derive(Debug, Default, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Square {
    pub index: u8,
}

impl Square {
    pub const fn new(rank: u8, file: u8) -> Self {
        assert!(rank <= 7, "rank is out of range");
        assert!(file <= 7, "file is out of range");

        let index = (rank * 8) + file;
        Self { index }
    }

    pub const fn index(index: u8) -> Self {
        Self { index }
    }

    #[inline]
    pub const fn rank(&self) -> u8 {
        (self.index / 8) as u8
    }

    #[inline]
    pub const fn file(&self) -> u8 {
        (self.index % 8) as u8
    }

    pub const fn in_board(&self) -> bool {
        let rank = self.rank() as usize;
        let file = self.file() as usize;

        let between_rank = rank >= Board::MIN_RANK && rank <= Board::MAX_RANK;
        let between_file = file >= Board::MIN_FILE && file <= Board::MAX_FILE;

        between_rank && between_file
    }

    #[inline]
    pub const fn get_pawn_pushes(&self, color: Color) -> Bitboard {
        let moves = tables::PAWN_PUSHES[color.index()][self.index as usize];
        Bitboard::bits(moves)
    }

    #[inline]
    pub fn get_pawn_attacks(&self, color: Color) -> Bitboard {
        let moves = tables::PAWN_ATTACKS[color.index()][self.index as usize];
        Bitboard::bits(moves)
    }

    #[inline]
    pub fn get_king_moves(&self) -> Bitboard {
        let moves = tables::KING_MOVES[self.index as usize];
        Bitboard::bits(moves)
    }

    #[inline]
    pub fn get_knight_moves(&self) -> Bitboard {
        let moves = tables::KNIGHT_MOVES[self.index as usize];
        Bitboard::bits(moves)
    }

    #[inline]
    pub fn get_ray(&self, direction: Direction) -> Bitboard {
        let bits = tables::RAYS[self.index as usize][direction.index()];
        Bitboard::bits(bits)
    }

    pub fn get_direction(&self, other: Square) -> Option<Direction> {
        let direction = tables::DIRECTION_LOOKUP[self.index as usize][other.index as usize];
        if direction == Direction::None {
            return None;
        }

        Some(direction)
    }

    #[inline]
    pub fn get_between(&self, other: Square) -> Bitboard {
        let bits = tables::BETWEEN_LOOKUP[self.index as usize][other.index as usize];
        Bitboard::bits(bits)
    }

    #[inline]
    pub fn get_bishop_mask(&self) -> Bitboard {
        let bits = tables::BISHOP_MASKS[self.index as usize];
        Bitboard::bits(bits)
    }

    #[inline]
    pub fn get_bishop_mask_ones(&self) -> usize {
        tables::BISHOP_MASK_ONES[self.index as usize]
    }

    #[inline]
    pub fn get_bishop_magic(&self) -> u64 {
        tables::BISHOP_MAGICS[self.index as usize]
    }

    #[inline]
    pub fn get_rook_mask(&self) -> Bitboard {
        let bits = tables::ROOK_MASKS[self.index as usize];
        Bitboard::bits(bits)
    }

    #[inline]
    pub fn get_rook_mask_ones(&self) -> usize {
        tables::ROOK_MASK_ONES[self.index as usize]
    }

    #[inline]
    pub fn get_rook_magic(&self) -> u64 {
        tables::ROOK_MAGICS[self.index as usize]
    }

    #[inline]
    pub fn get_rook_attacks(&self, occupancy: Bitboard) -> Bitboard {
        let mask = self.get_rook_mask();
        let blockers = occupancy & mask;

        let magic = self.get_rook_magic();
        let ones = self.get_rook_mask_ones();

        let index = (blockers.bits.wrapping_mul(magic) >> (64 - ones)) as usize;
        let attacks = tables::ROOK_ATTACKS[self.index as usize][index];
        Bitboard::bits(attacks)
    }

    #[inline]
    pub fn get_bishop_attacks(&self, occupancy: Bitboard) -> Bitboard {
        let mask = self.get_bishop_mask();
        let blockers = occupancy & mask;

        let magic = self.get_bishop_magic();
        let ones = self.get_bishop_mask_ones();

        let index = (blockers.bits.wrapping_mul(magic) >> (64 - ones)) as usize;
        let attacks = tables::BISHOP_ATTACKS[self.index as usize][index];
        Bitboard::bits(attacks)
    }

    pub const fn get_midgame_value(&self, color: Color, piece: Piece) -> isize {
        let index = match color {
            Color::White => FLIP[self.index as usize],
            Color::Black => self.index as usize,
        };

        match piece {
            Piece::Pawn => MIDGAME_PAWN_TABLE[index],
            Piece::Knight => MIDGAME_KNIGHT_TABLE[index],
            Piece::Bishop => MIDGAME_BISHOP_TABLE[index],
            Piece::Rook => MIDGAME_ROOK_TABLE[index],
            Piece::Queen => MIDGAME_QUEEN_TABLE[index],
            Piece::King => MIDGAME_KING_TABLE[index],
            Piece::None => 0,
        }
    }

    pub const fn get_endgame_value(&self, color: Color, piece: Piece) -> isize {
        let index = match color {
            Color::White => FLIP[self.index as usize],
            Color::Black => self.index as usize,
        };

        match piece {
            Piece::Pawn => ENDGAME_PAWN_TABLE[index],
            Piece::Knight => ENDGAME_KNIGHT_TABLE[index],
            Piece::Bishop => ENDGAME_BISHOP_TABLE[index],
            Piece::Rook => ENDGAME_ROOK_TABLE[index],
            Piece::Queen => ENDGAME_QUEEN_TABLE[index],
            Piece::King => ENDGAME_KING_TABLE[index],
            Piece::None => 0,
        }
    }

    pub fn is_attacked(&self, mov: &Move) -> bool {
        if !mov.is_capture() {
            return false;
        }

        let capture_square = mov.capture_square();
        capture_square == *self
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rank = self.rank() + 1;
        let file = (b'a' + self.file()) as char;

        write!(f, "{}{}", file, rank)
    }
}

impl FromStr for Square {
    type Err = SquareError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut stream = input.chars();

        let file = match stream.next() {
            Some(char) if char.is_ascii_lowercase() => char as u8 - b'a',
            _ => return Err(InvalidSquareFormat::new(input).into()),
        };
        let rank = match stream.next() {
            Some(char) if char.is_digit(10) => (char as u8 - b'0') - 1,
            _ => return Err(InvalidSquareFormat::new(input).into()),
        };

        let square = Self::new(rank, file);
        Ok(square)
    }
}

impl From<u8> for Square {
    fn from(value: u8) -> Self {
        Self::index(value)
    }
}
