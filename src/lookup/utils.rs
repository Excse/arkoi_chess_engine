use std::cmp::Ordering;

use strum::{EnumIter, IntoEnumIterator};

use crate::{bitboard::Square, board::Board};

#[derive(Debug, Clone, Copy, EnumIter)]
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
        *self as usize
    }

    pub fn at(index: usize) -> Option<Self> {
        Direction::iter().nth(index)
    }
}

pub fn get_direction_index(from: Square, to: Square) -> Option<usize> {
    let rank_cmp = to.rank.cmp(&from.rank);
    let file_cmp = to.file.cmp(&from.file);
    if rank_cmp.is_eq() && file_cmp.is_eq() {
        return None;
    }

    let rank_diff = to.rank as i8 - from.rank as i8;
    let file_diff = to.file as i8 - from.file as i8;
    let equal_delta = rank_diff.abs() == file_diff.abs();

    return Some(match (rank_cmp, file_cmp, equal_delta) {
        (Ordering::Greater, Ordering::Less, true) => 0,
        (Ordering::Greater, Ordering::Equal, false) => 1,
        (Ordering::Greater, Ordering::Greater, true) => 2,

        (Ordering::Equal, Ordering::Less, false) => 3,
        (Ordering::Equal, Ordering::Greater, false) => 4,

        (Ordering::Less, Ordering::Less, true) => 5,
        (Ordering::Less, Ordering::Equal, false) => 6,
        (Ordering::Less, Ordering::Greater, true) => 7,

        _ => return None,
    });
}

pub fn inside_board(rank: i8, file: i8) -> bool {
    let between_rank = rank >= Board::MIN_RANK as i8 && rank <= Board::MAX_RANK as i8;
    let between_file = file >= Board::MIN_FILE as i8 && file <= Board::MAX_FILE as i8;
    between_rank && between_file
}
