use std::{fmt::Display, str::FromStr};

use crate::board::Board;

use super::error::{InvalidSquareFormat, SquareError};

#[derive(Debug, Default, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Square {
    pub rank: u8,
    pub file: u8,
    pub index: usize,
}

impl Square {
    pub fn new(rank: u8, file: u8) -> Self {
        let index = (rank * 8) + file;
        Self {
            rank,
            file,
            index: index as usize,
        }
    }

    pub const fn index(index: usize) -> Self {
        let rank = (index / 8) as u8;
        let file = (index % 8) as u8;
        Self { rank, file, index }
    }

    pub fn in_board(&self) -> bool {
        let rank = self.rank as usize;
        let file = self.file as usize;

        let between_rank = rank >= Board::MIN_RANK && rank <= Board::MAX_RANK;
        let between_file = file >= Board::MIN_FILE && file <= Board::MAX_FILE;

        between_rank && between_file
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rank = self.rank + 1;
        let file = (b'a' + self.file) as char;

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
