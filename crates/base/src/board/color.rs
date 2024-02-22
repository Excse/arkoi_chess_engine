#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black,
    White,
}

impl Color {
    pub const COUNT: usize = 2;

    #[inline(always)]
    pub const fn index(&self) -> usize {
        *self as usize
    }

    #[inline(always)]
    pub const fn en_passant_offset(&self) -> i8 {
        match self {
            Self::White => -8,
            Self::Black => 8,
        }
    }

    #[inline(always)]
    pub const fn other(&self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }
}
