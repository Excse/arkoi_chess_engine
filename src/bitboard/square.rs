use std::{fmt::Display, str::FromStr};

use crate::{
    board::{color::Color, piece::Piece, Board},
    generation::mov::Move,
    lookup::{pesto::*, tables, utils::Direction},
};

use super::{
    error::{InvalidSquareFormat, SquareError},
    Bitboard,
};

#[derive(Debug, Default, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Square(u8);

impl Square {
    pub const fn new(rank: u8, file: u8) -> Self {
        debug_assert!(rank <= 7, "rank is out of range");
        debug_assert!(file <= 7, "file is out of range");

        let index = (rank * 8) + file;
        Self(index)
    }

    pub const fn by_index(index: u8) -> Self {
        debug_assert!(index <= 63, "index is out of range");

        Self(index)
    }

    #[inline(always)]
    pub const fn rank(&self) -> u8 {
        self.0 / 8
    }

    #[inline(always)]
    pub const fn file(&self) -> u8 {
        self.0 % 8
    }

    pub const fn in_board(&self) -> bool {
        let rank = self.rank() as usize;
        let file = self.file() as usize;

        let between_rank = rank >= Board::MIN_RANK && rank <= Board::MAX_RANK;
        let between_file = file >= Board::MIN_FILE && file <= Board::MAX_FILE;

        between_rank && between_file
    }

    #[inline(always)]
    pub fn get_relative_index(&self, color: Color) -> usize {
        match color {
            Color::White => unsafe { *FLIP.get_unchecked(self.0 as usize) },
            Color::Black => self.0 as usize,
        }
    }

    #[inline(always)]
    pub fn get_pawn_pushes(&self, color: Color) -> Bitboard {
        unsafe {
            let squares = tables::PAWN_PUSHES.get_unchecked(color.index());
            let push = squares.get_unchecked(self.0 as usize);
            Bitboard::bits(*push)
        }
    }

    #[inline(always)]
    pub fn get_pawn_attacks(&self, color: Color) -> Bitboard {
        unsafe {
            let squares = tables::PAWN_ATTACKS.get_unchecked(color.index());
            let attacks = squares.get_unchecked(self.0 as usize);
            Bitboard::bits(*attacks)
        }
    }

    #[inline(always)]
    pub fn get_king_moves(&self) -> Bitboard {
        unsafe {
            let moves = tables::KING_MOVES.get_unchecked(self.0 as usize);
            Bitboard::bits(*moves)
        }
    }

    #[inline(always)]
    pub fn get_knight_moves(&self) -> Bitboard {
        unsafe {
            let moves = tables::KNIGHT_MOVES.get_unchecked(self.0 as usize);
            Bitboard::bits(*moves)
        }
    }

    #[inline(always)]
    pub fn get_ray(&self, direction: Direction) -> Bitboard {
        unsafe {
            let rays = tables::RAYS.get_unchecked(self.0 as usize);
            let ray = rays.get_unchecked(direction.index());
            Bitboard::bits(*ray)
        }
    }

    pub fn get_direction(&self, other: Square) -> Option<Direction> {
        let direction = tables::DIRECTION_LOOKUP[self.0 as usize][other.0 as usize];
        if direction == Direction::None {
            return None;
        }

        Some(direction)
    }

    #[inline(always)]
    pub fn get_between(&self, other: Square) -> Bitboard {
        unsafe {
            let squares = tables::BETWEEN_LOOKUP.get_unchecked(self.0 as usize);
            let bits = squares.get_unchecked(other.0 as usize);
            Bitboard::bits(*bits)
        }
    }

    #[inline(always)]
    pub fn get_bishop_mask(&self) -> Bitboard {
        unsafe {
            let mask = tables::BISHOP_MASKS.get_unchecked(self.0 as usize);
            Bitboard::bits(*mask)
        }
    }

    #[inline(always)]
    pub fn get_bishop_mask_ones(&self) -> usize {
        unsafe {
            let mask_ones = tables::BISHOP_MASK_ONES.get_unchecked(self.0 as usize);
            *mask_ones
        }
    }

    #[inline(always)]
    pub fn get_bishop_magic(&self) -> u64 {
        unsafe {
            let magic = tables::BISHOP_MAGICS.get_unchecked(self.0 as usize);
            *magic
        }
    }

    #[inline(always)]
    pub fn get_rook_mask(&self) -> Bitboard {
        unsafe {
            let mask = tables::ROOK_MASKS.get_unchecked(self.0 as usize);
            Bitboard::bits(*mask)
        }
    }

    #[inline(always)]
    pub fn get_rook_mask_ones(&self) -> usize {
        unsafe {
            let mask_ones = tables::ROOK_MASK_ONES.get_unchecked(self.0 as usize);
            *mask_ones
        }
    }

    #[inline(always)]
    pub fn get_rook_magic(&self) -> u64 {
        unsafe {
            let magic = tables::ROOK_MAGICS.get_unchecked(self.0 as usize);
            *magic
        }
    }

    #[inline]
    pub fn get_rook_attacks(&self, occupancy: Bitboard) -> Bitboard {
        let mask = self.get_rook_mask();
        let blockers = occupancy & mask;

        let magic = self.get_rook_magic();
        let ones = self.get_rook_mask_ones();

        let index = (blockers.bits.wrapping_mul(magic) >> (64 - ones)) as usize;

        unsafe {
            let magics = tables::ROOK_ATTACKS.get_unchecked(self.0 as usize);
            let attacks = magics.get_unchecked(index);
            Bitboard::bits(*attacks)
        }
    }

    #[inline]
    pub fn get_bishop_attacks(&self, occupancy: Bitboard) -> Bitboard {
        let mask = self.get_bishop_mask();
        let blockers = occupancy & mask;

        let magic = self.get_bishop_magic();
        let ones = self.get_bishop_mask_ones();

        let index = (blockers.bits.wrapping_mul(magic) >> (64 - ones)) as usize;

        unsafe {
            let magics = tables::BISHOP_ATTACKS.get_unchecked(self.0 as usize);
            let attacks = magics.get_unchecked(index);
            Bitboard::bits(*attacks)
        }
    }

    #[inline]
    pub fn get_midgame_value(&self, color: Color, piece: Piece) -> isize {
        let index = self.get_relative_index(color);

        unsafe {
            match piece {
                Piece::Pawn => *MIDGAME_PAWN_TABLE.get_unchecked(index),
                Piece::Knight => *MIDGAME_KNIGHT_TABLE.get_unchecked(index),
                Piece::Bishop => *MIDGAME_BISHOP_TABLE.get_unchecked(index),
                Piece::Rook => *MIDGAME_ROOK_TABLE.get_unchecked(index),
                Piece::Queen => *MIDGAME_QUEEN_TABLE.get_unchecked(index),
                Piece::King => *MIDGAME_KING_TABLE.get_unchecked(index),
                Piece::None => 0,
            }
        }
    }

    #[inline]
    pub fn get_endgame_value(&self, color: Color, piece: Piece) -> isize {
        let index = self.get_relative_index(color);

        unsafe {
            match piece {
                Piece::Pawn => *ENDGAME_PAWN_TABLE.get_unchecked(index),
                Piece::Knight => *ENDGAME_KNIGHT_TABLE.get_unchecked(index),
                Piece::Bishop => *ENDGAME_BISHOP_TABLE.get_unchecked(index),
                Piece::Rook => *ENDGAME_ROOK_TABLE.get_unchecked(index),
                Piece::Queen => *ENDGAME_QUEEN_TABLE.get_unchecked(index),
                Piece::King => *ENDGAME_KING_TABLE.get_unchecked(index),
                Piece::None => 0,
            }
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
        Self::by_index(value)
    }
}

impl From<Square> for usize {
    fn from(value: Square) -> Self {
        value.0 as usize
    }
}

impl From<Square> for isize {
    fn from(value: Square) -> Self {
        value.0 as isize
    }
}

impl From<Square> for u64 {
    fn from(value: Square) -> Self {
        value.0 as u64
    }
}

impl From<Square> for u8 {
    fn from(value: Square) -> Self {
        value.0 as u8
    }
}

impl From<Square> for i8 {
    fn from(value: Square) -> Self {
        value.0 as i8
    }
}
