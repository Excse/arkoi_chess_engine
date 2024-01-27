use std::cmp::Ordering;

use crate::{bitboard::Square, board::Board};

use super::tables::RAYS;

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    NorthWest,
    North,
    NorthEast,
    West,
    East,
    SouthWest,
    South,
    SouthEast,
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
        }
    }

    pub fn between(first: Square, second: Square) -> Option<Direction> {
        let rank_cmp = second.rank.cmp(&first.rank);
        let file_cmp = second.file.cmp(&first.file);
        if rank_cmp.is_eq() && file_cmp.is_eq() {
            return None;
        }

        let rank_diff = second.rank as i8 - first.rank as i8;
        let file_diff = second.file as i8 - first.file as i8;
        let equal_delta = rank_diff.abs() == file_diff.abs();

        return Some(match (rank_cmp, file_cmp, equal_delta) {
            (Ordering::Greater, Ordering::Less, true) => Direction::NorthWest,
            (Ordering::Greater, Ordering::Equal, false) => Direction::North,
            (Ordering::Greater, Ordering::Greater, true) => Direction::NorthEast,

            (Ordering::Equal, Ordering::Less, false) => Direction::West,
            (Ordering::Equal, Ordering::Greater, false) => Direction::East,

            (Ordering::Less, Ordering::Less, true) => Direction::SouthWest,
            (Ordering::Less, Ordering::Equal, false) => Direction::South,
            (Ordering::Less, Ordering::Greater, true) => Direction::SouthEast,

            _ => return None,
        });
    }
}

pub fn inside_board(rank: i8, file: i8) -> bool {
    let between_rank = rank >= Board::MIN_RANK as i8 && rank <= Board::MAX_RANK as i8;
    let between_file = file >= Board::MIN_FILE as i8 && file <= Board::MAX_FILE as i8;
    between_rank && between_file
}
