use std::ops::Not;

use strum::{EnumIter, IntoEnumIterator};

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
