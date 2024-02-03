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

    pub const fn index(&self) -> usize {
        match self {
            Self::NorthWest => 0,
            Self::North => 1,
            Self::NorthEast => 2,
            Self::West => 3,
            Self::East => 4,
            Self::SouthWest => 5,
            Self::South => 6,
            Self::SouthEast => 7,
            _ => panic!("Invalid direction"),
        }
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
