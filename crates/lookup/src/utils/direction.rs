use super::utils::rank_file;

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
    pub const fn is_none(&self) -> bool {
        self.index() == 8
    }

    #[inline(always)]
    pub const fn index(&self) -> usize {
        *self as usize
    }

    pub const fn from_index(index: usize) -> Self {
        match index {
            0 => Self::NorthWest,
            1 => Self::North,
            2 => Self::NorthEast,
            3 => Self::West,
            4 => Self::East,
            5 => Self::SouthWest,
            6 => Self::South,
            7 => Self::SouthEast,
            _ => Self::None,
        }
    }

    pub const fn opposite(&self) -> Self {
        match self {
            Self::NorthWest => Self::SouthEast,
            Self::North => Self::South,
            Self::NorthEast => Self::SouthWest,
            Self::West => Self::East,
            Self::East => Self::West,
            Self::SouthWest => Self::NorthEast,
            Self::South => Self::North,
            Self::SouthEast => Self::NorthWest,
            Self::None => Self::None,
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

    #[rustfmt::skip]
    pub(crate) const fn between(from: usize, to: usize) -> Direction {
        let (from_rank, from_file) = rank_file(from);
        let (to_rank, to_file) = rank_file(to);

        let rank_diff = to_rank as i8 - from_rank as i8;
        let file_diff = to_file as i8 - from_file as i8;
        
        let is_straight = rank_diff == 0 || file_diff == 0;
        if is_straight {
            if file_diff > 0 { return Direction::East; }
            if file_diff < 0 { return Direction::West; }
            if rank_diff > 0 { return Direction::North; }
            if rank_diff < 0 { return Direction::South; }
        }
        
        let is_diagonal = rank_diff.abs() == file_diff.abs();
        if is_diagonal {
            if rank_diff > 0 && file_diff > 0 { return Direction::NorthEast; }
            if rank_diff > 0 && file_diff < 0 { return Direction::NorthWest; }
            if rank_diff < 0 && file_diff > 0 { return Direction::SouthEast; }
            if rank_diff < 0 && file_diff < 0 { return Direction::SouthWest; }
        }

        Direction::None
    }
}
