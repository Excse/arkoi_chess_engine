use const_for::*;

use crate::{
    direction::Direction,
    utils::{bits, index, rank_file},
    BOARD_SIZE,
};

pub(crate) const INVALID_SQUARE: usize = 64;

pub(crate) type TableMove = (i8, i8);

#[rustfmt::skip]
pub(crate) const DIRECTION_MOVES: [TableMove; 8] = [
    ( 1, -1), ( 1, 0), ( 1, 1),
    ( 0, -1),          ( 0, 1),
    (-1, -1), (-1, 0), (-1, 1),
];

#[rustfmt::skip]
pub const BETWEEN: [[u64; BOARD_SIZE]; BOARD_SIZE] = {
    let mut between = [[0; BOARD_SIZE]; BOARD_SIZE];

    const_for!(from in 0..BOARD_SIZE => {
        const_for!(to in 0..BOARD_SIZE => {
            let direction = Direction::between(from, to);
            if direction.is_none() {
                continue;
            }

            let ray_between = ray(from, direction, to);
            between[from][to] |= ray_between;
        });
    });

    between
};

#[rustfmt::skip]
pub const LINE: [[u64; BOARD_SIZE]; BOARD_SIZE] = {
    let mut line = [[0; BOARD_SIZE]; BOARD_SIZE];

    const_for!(from in 0..BOARD_SIZE => {
        const_for!(to in 0..BOARD_SIZE => {
            let direction = Direction::between(from, to);
            if direction.is_none() {
                continue;
            }

            let ray_line = ray(from, direction, INVALID_SQUARE);
            line[from][to] |= ray_line;

            let opposite = direction.opposite();

            let ray_line = ray(from, opposite, INVALID_SQUARE);
            line[from][to] |= ray_line;

            let (rank, file) = rank_file(from);
            line[from][to] |= bits(rank, file);
        });
    });

    line
};

#[rustfmt::skip]
pub const ADJACENT_FILE_SQUARES: [u64; BOARD_SIZE] = {
    let mut adjacent_files = [0; BOARD_SIZE];

    const_for!(rank in 0..8 => {
        const_for!(file in 0..8 => {
            let from = index(rank, file);
            adjacent_files[from] |= bits(rank, file);
            
            if file > 0 { adjacent_files[from] |= bits(rank, file - 1); }
            if file < 7 { adjacent_files[from] |= bits(rank, file + 1); }
        });
    });

    adjacent_files
};

#[rustfmt::skip]
pub const RAYS: [[u64; 8]; BOARD_SIZE] = {
    let mut rays = [[0; 8]; BOARD_SIZE];

    const_for!(from in 0..BOARD_SIZE => {
        const_for!(direction in 0..8 => {
            let ray = ray(from, Direction::from_index(direction), INVALID_SQUARE);
            rays[from][direction] = ray;
        });
    });

    rays
};

const fn ray(from: usize, direction: Direction, collide: usize) -> u64 {
    if from == collide || direction.is_none() {
        return 0;
    }

    let (delta_rank, delta_file) = DIRECTION_MOVES[direction.index()];
    let (mut current_rank, mut current_file) = rank_file(from);

    let mut ray = 0;

    loop {
        current_rank = (current_rank as i8 + delta_rank) as usize;
        current_file = (current_file as i8 + delta_file) as usize;

        if current_rank > 7 || current_file > 7 {
            break;
        }

        if index(current_rank, current_file) == collide {
            break;
        }

        ray |= bits(current_rank, current_file);
    }

    ray
}
