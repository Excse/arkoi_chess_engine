use std::{fmt::Display, str::FromStr};

use crate::{
    board::{color::Color, Board},
    lookup::{tables, utils::Direction},
};

use super::{
    error::{InvalidSquareFormat, SquareError},
    Bitboard,
};

#[derive(Debug, Default, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Square {
    pub index: usize,
}

impl Square {
    pub const fn new(rank: u8, file: u8) -> Self {
        let index = ((rank * 8) + file) as usize;
        assert!(rank <= 7, "rank is out of range");
        assert!(file <= 7, "file is out of range");

        Self { index }
    }

    pub const fn index(index: usize) -> Self {
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
        let moves = tables::PAWN_PUSHES[color.index()][self.index];
        Bitboard::bits(moves)
    }

    #[inline]
    pub fn get_pawn_attacks(&self, color: Color) -> Bitboard {
        let moves = tables::PAWN_ATTACKS[color.index()][self.index];
        Bitboard::bits(moves)
    }

    #[inline]
    pub fn get_king_moves(&self) -> Bitboard {
        let moves = tables::KING_MOVES[self.index];
        Bitboard::bits(moves)
    }

    #[inline]
    pub fn get_knight_moves(&self) -> Bitboard {
        let moves = tables::KNIGHT_MOVES[self.index];
        Bitboard::bits(moves)
    }

    #[inline]
    pub fn get_ray(&self, direction: Direction) -> Bitboard {
        let bits = tables::RAYS[self.index][direction.index()];
        Bitboard::bits(bits)
    }

    pub fn get_direction(&self, other: Square) -> Option<Direction> {
        let direction = tables::DIRECTION_LOOKUP[self.index][other.index];
        if direction == Direction::None {
            return None;
        }

        Some(direction)
    }

    #[inline]
    pub fn get_between(&self, other: Square) -> Bitboard {
        let bits = tables::BETWEEN_LOOKUP[self.index][other.index];
        Bitboard::bits(bits)
    }

    #[inline]
    pub fn get_bishop_mask(&self) -> Bitboard {
        let bits = tables::BISHOP_MASKS[self.index];
        Bitboard::bits(bits)
    }

    #[inline]
    pub fn get_bishop_mask_ones(&self) -> usize {
        tables::BISHOP_MASK_ONES[self.index]
    }

    #[inline]
    pub fn get_bishop_magic(&self) -> u64 {
        tables::BISHOP_MAGICS[self.index]
    }

    #[inline]
    pub fn get_rook_mask(&self) -> Bitboard {
        let bits = tables::ROOK_MASKS[self.index];
        Bitboard::bits(bits)
    }

    #[inline]
    pub fn get_rook_mask_ones(&self) -> usize {
        tables::ROOK_MASK_ONES[self.index]
    }

    #[inline]
    pub fn get_rook_magic(&self) -> u64 {
        tables::ROOK_MAGICS[self.index]
    }

    #[inline]
    pub fn get_rook_attacks(&self, occupancy: Bitboard) -> Bitboard {
        let mask = self.get_rook_mask();
        let blockers = occupancy & mask;

        let magic = self.get_rook_magic();
        let ones = self.get_rook_mask_ones();

        let index = (blockers.bits.wrapping_mul(magic) >> (64 - ones)) as usize;
        let attacks = tables::ROOK_ATTACKS[self.index][index];
        Bitboard::bits(attacks)
    }

    #[inline]
    pub fn get_bishop_attacks(&self, occupancy: Bitboard) -> Bitboard {
        let mask = self.get_bishop_mask();
        let blockers = occupancy & mask;

        let magic = self.get_bishop_magic();
        let ones = self.get_bishop_mask_ones();

        let index = (blockers.bits.wrapping_mul(magic) >> (64 - ones)) as usize;
        let attacks = tables::BISHOP_ATTACKS[self.index][index];
        Bitboard::bits(attacks)
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

impl From<usize> for Square {
    fn from(value: usize) -> Self {
        Self::index(value)
    }
}
