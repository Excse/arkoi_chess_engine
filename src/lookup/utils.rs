use crate::board::Board;

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

    pub fn index(&self) -> usize {
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

    pub fn is_diagonal(&self) -> bool {
        match self {
            Self::NorthWest | Self::NorthEast | Self::SouthWest | Self::SouthEast => true,
            _ => false,
        }
    }

    pub fn is_straight(&self) -> bool {
        match self {
            Self::North | Self::West | Self::East | Self::South => true,
            _ => false,
        }
    }

    pub fn is_horizontal(&self) -> bool {
        match self {
            Self::West | Self::East => true,
            _ => false,
        }
    }
}

pub fn inside_board(rank: i8, file: i8) -> bool {
    let between_rank = rank >= Board::MIN_RANK as i8 && rank <= Board::MAX_RANK as i8;
    let between_file = file >= Board::MIN_FILE as i8 && file <= Board::MAX_FILE as i8;
    between_rank && between_file
}
