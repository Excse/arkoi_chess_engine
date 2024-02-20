use std::{fmt::Display, str::FromStr};

use crate::{
    bitboard::{
        constants::{FILES, RANKS},
        Bitboard,
    },
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
