use crate::{
    utils::{
        direction::Direction,
        utils::{bits, index, rank_file},
    },
    BOARD_SIZE,
};

const INVALID_SQUARE: usize = 64;

#[rustfmt::skip]
const DIRECTION_MOVES: [(i8, i8); 8] = [
    ( 1, -1), ( 1, 0), ( 1, 1),
    ( 0, -1),          ( 0, 1),
    (-1, -1), (-1, 0), (-1, 1),
];

#[rustfmt::skip]
pub fn generate_between() -> [[u64; BOARD_SIZE]; BOARD_SIZE] {
    let mut between = [[0; BOARD_SIZE]; BOARD_SIZE];

    for from in 0..BOARD_SIZE {
        for to in 0..BOARD_SIZE {
            let direction = Direction::between(from, to);
            if direction.is_none() {
                continue;
            }

            let ray_between = ray(from, direction, to);
            between[from][to] |= ray_between;
        }
    }

    between
}

#[rustfmt::skip]
pub fn generate_lines() -> [[u64; BOARD_SIZE]; BOARD_SIZE] {
    let mut lines = [[0; BOARD_SIZE]; BOARD_SIZE];

    for from in 0..BOARD_SIZE {
        for to in 0..BOARD_SIZE {
            let direction = Direction::between(from, to);
            if direction.is_none() {
                continue;
            }

            let ray_line = ray(from, direction, INVALID_SQUARE);
            lines[from][to] |= ray_line;

            let opposite = direction.opposite();

            let ray_line = ray(from, opposite, INVALID_SQUARE);
            lines[from][to] |= ray_line;

            let (rank, file) = rank_file(from);
            lines[from][to] |= bits(rank, file);
        }
    }

    lines
}

#[rustfmt::skip]
pub fn generate_adjacent_files() -> [u64; BOARD_SIZE]  {
    let mut adjacent_files = [0; BOARD_SIZE];

    for rank in 0..8 {
        for file in 0..8 {
            let from = index(rank, file);
            adjacent_files[from] |= bits(rank, file);
            
            if file > 0 { adjacent_files[from] |= bits(rank, file - 1); }
            if file < 7 { adjacent_files[from] |= bits(rank, file + 1); }
        }
    }

    adjacent_files
}

#[rustfmt::skip]
pub fn generate_rays() -> [[u64; 8]; BOARD_SIZE] {
    let mut rays = [[0; 8]; BOARD_SIZE];

    for from in 0..BOARD_SIZE {
        for direction in 0..8 {
            let ray = ray(from, Direction::from_index(direction), INVALID_SQUARE);
            rays[from][direction] = ray;
        }
    }

    rays
}

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
