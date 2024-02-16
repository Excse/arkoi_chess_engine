#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    NorthWest,
    North,
    NorthEast,
    West,
    East,
    SouthWest,
    South,
    SouthEast,
    None,
}

impl Direction {
    pub const COUNT: usize = 8;

    #[inline(always)]
    pub fn index(&self) -> usize {
        debug_assert!(*self != Self::None, "Invalid direction");

        *self as usize
    }

    pub const fn is_diagonal(&self) -> bool {
        match self {
            Self::NorthWest | Self::NorthEast | Self::SouthWest | Self::SouthEast => true,
            _ => false,
        }
    }

    pub const fn is_straight(&self) -> bool {
        match self {
            Self::North | Self::West | Self::East | Self::South => true,
            _ => false,
        }
    }

    pub const fn is_horizontal(&self) -> bool {
        match self {
            Self::West | Self::East => true,
            _ => false,
        }
    }
}
