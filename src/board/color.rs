use std::ops::Not;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black,
    White,
}

impl Color {
    pub const COUNT: usize = 2;

    pub const fn index(&self) -> usize {
        *self as usize
    }

    pub const fn en_passant_offset(&self) -> i8 {
        match self {
            Self::White => -8,
            Self::Black => 8,
        }
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
