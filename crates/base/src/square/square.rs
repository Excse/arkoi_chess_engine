use std::{fmt::Display, str::FromStr};

use lookup::{generated::*, pesto::*, utils::direction::Direction};

use crate::{
    bitboard::{
        constants::{FILES, RANKS},
        Bitboard,
    },
    board::{color::Color, piece::Piece},
    r#move::Move,
};

use super::error::{InvalidSquareFormat, SquareError};

#[derive(Debug, Default, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Square(u8);

impl Square {
    pub const fn new(rank: u8, file: u8) -> Self {
        debug_assert!(rank <= 7);
        debug_assert!(file <= 7);

        let index = (rank * 8) + file;
        Self(index)
    }

    pub const fn from_index(index: u8) -> Self {
        debug_assert!(index <= 63);

        Self(index)
    }

    #[inline(always)]
    pub const fn index(&self) -> u8 {
        self.0
    }
}

impl Square {
    #[inline(always)]
    pub const fn rank(&self) -> u8 {
        self.0 / 8
    }

    #[inline(always)]
    pub fn rank_bb(&self) -> Bitboard {
        unsafe {
            let rank = RANKS.get_unchecked(self.rank() as usize);
            *rank
        }
    }

    #[inline(always)]
    pub const fn file(&self) -> u8 {
        self.0 % 8
    }

    #[inline(always)]
    pub fn file_bb(&self) -> Bitboard {
        unsafe {
            let file = FILES.get_unchecked(self.rank() as usize);
            *file
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

impl Square {
    #[inline(always)]
    pub fn get_adjacent_files(&self) -> Bitboard {
        unsafe {
            let adjacent = ADJACENT_FILES.get_unchecked(self.0 as usize);
            Bitboard::from_bits(*adjacent)
        }
    }

    #[inline(always)]
    pub fn get_pawn_pushes(&self, color: Color) -> Bitboard {
        debug_assert!(color.index() < Color::COUNT);

        unsafe {
            let squares = PAWN_PUSHES.get_unchecked(color.index());
            let push = squares.get_unchecked(self.0 as usize);
            Bitboard::from_bits(*push)
        }
    }

    #[inline(always)]
    pub fn get_pawn_attacks(&self, color: Color) -> Bitboard {
        debug_assert!(color.index() < Color::COUNT);

        unsafe {
            let squares = PAWN_ATTACKS.get_unchecked(color.index());
            let attacks = squares.get_unchecked(self.0 as usize);
            Bitboard::from_bits(*attacks)
        }
    }

    #[inline(always)]
    pub fn get_king_moves(&self) -> Bitboard {
        unsafe {
            let moves = KING_MOVES.get_unchecked(self.0 as usize);
            Bitboard::from_bits(*moves)
        }
    }

    #[inline(always)]
    pub fn get_knight_moves(&self) -> Bitboard {
        unsafe {
            let moves = KNIGHT_MOVES.get_unchecked(self.0 as usize);
            Bitboard::from_bits(*moves)
        }
    }

    #[inline(always)]
    pub fn get_ray(&self, direction: Direction) -> Bitboard {
        debug_assert!(direction != Direction::None);

        unsafe {
            let rays = RAYS.get_unchecked(self.0 as usize);
            let ray = rays.get_unchecked(direction.index());
            Bitboard::from_bits(*ray)
        }
    }

    #[inline(always)]
    pub fn get_between(&self, other: Square) -> Bitboard {
        unsafe {
            let squares = BETWEEN.get_unchecked(self.0 as usize);
            let bits = squares.get_unchecked(other.0 as usize);
            Bitboard::from_bits(*bits)
        }
    }

    #[inline(always)]
    pub fn get_line(&self, other: Square) -> Bitboard {
        unsafe {
            let squares = LINES.get_unchecked(self.0 as usize);
            let bits = squares.get_unchecked(other.0 as usize);
            Bitboard::from_bits(*bits)
        }
    }

    #[inline(always)]
    pub fn get_bishop_mask(&self) -> Bitboard {
        unsafe {
            let mask = BISHOP_MASKS.get_unchecked(self.0 as usize);
            Bitboard::from_bits(*mask)
        }
    }

    #[inline(always)]
    pub fn get_bishop_mask_ones(&self) -> u32 {
        unsafe {
            let mask_ones = BISHOP_MASK_ONES.get_unchecked(self.0 as usize);
            *mask_ones
        }
    }

    #[inline(always)]
    pub fn get_bishop_magic(&self) -> u64 {
        unsafe {
            let magic = BISHOP_MAGICS.get_unchecked(self.0 as usize);
            *magic
        }
    }

    #[inline(always)]
    pub fn get_rook_mask(&self) -> Bitboard {
        unsafe {
            let mask = ROOK_MASKS.get_unchecked(self.0 as usize);
            Bitboard::from_bits(*mask)
        }
    }

    #[inline(always)]
    pub fn get_rook_mask_ones(&self) -> u32 {
        unsafe {
            let mask_ones = ROOK_MASK_ONES.get_unchecked(self.0 as usize);
            *mask_ones
        }
    }

    #[inline(always)]
    pub fn get_rook_magic(&self) -> u64 {
        unsafe {
            let magic = ROOK_MAGICS.get_unchecked(self.0 as usize);
            *magic
        }
    }

    #[inline]
    pub fn get_rook_attacks(&self, occupancy: Bitboard) -> Bitboard {
        let mask = self.get_rook_mask();
        let blockers = occupancy & mask;

        let magic = self.get_rook_magic();
        let ones = self.get_rook_mask_ones();

        let magic_index = blockers.get_magic_index(magic, ones);
        unsafe {
            let magics = ROOK_MAGIC_ATTACKS.get_unchecked(self.0 as usize);
            let attacks = magics.get_unchecked(magic_index);
            Bitboard::from_bits(*attacks)
        }
    }

    #[inline]
    pub fn get_bishop_attacks(&self, occupancy: Bitboard) -> Bitboard {
        let mask = self.get_bishop_mask();
        let blockers = occupancy & mask;

        let magic = self.get_bishop_magic();
        let ones = self.get_bishop_mask_ones();

        let magic_index = blockers.get_magic_index(magic, ones);
        unsafe {
            let magics = BISHOP_MAGIC_ATTACKS.get_unchecked(self.0 as usize);
            let attacks = magics.get_unchecked(magic_index);
            Bitboard::from_bits(*attacks)
        }
    }

    #[inline(always)]
    pub fn get_relative_index(&self, color: Color) -> usize {
        match color {
            Color::White => unsafe { *FLIP.get_unchecked(self.0 as usize) },
            Color::Black => self.0 as usize,
        }
    }

    #[inline]
    pub fn get_midgame_value(&self, color: Color, piece: Piece) -> i32 {
        let index = self.get_relative_index(color);
        debug_assert!(index < 64);

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
    pub fn get_endgame_value(&self, color: Color, piece: Piece) -> i32 {
        let index = self.get_relative_index(color);
        debug_assert!(index < 64);

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
        Self::from_index(value)
    }
}

impl From<Bitboard> for Square {
    fn from(value: Bitboard) -> Self {
        debug_assert_eq!(value.count_ones(), 1, "bitboard must have only one bit set");

        let index = value.get_trailing_index();
        Self::from_index(index)
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
