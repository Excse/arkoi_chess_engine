use std::cmp::Ordering;

use crate::{
    bitboard::{square::Square, Bitboard},
    board::Board,
};

use self::utils::{inside_board, Direction};

pub mod tables;
pub mod utils;

// Defines a relative move in the format of (rank, file)
type TableMove = (i8, i8);

// ------------- ALL MOVES TO GENERATE THE TABLE -------------

#[rustfmt::skip]
const KING_MOVES: [TableMove; 8] = [
    ( 1, -1), ( 1, 0), ( 1, 1),
    ( 0, -1),          ( 0, 1),
    (-1, -1), (-1, 0), (-1, 1),
];

const WHITE_PAWN_MOVE: [TableMove; 1] = [(1, 0)];
const BLACK_PAWN_MOVE: [TableMove; 1] = [(-1, 0)];

const WHITE_PAWN_ATTACKS: [TableMove; 2] = [(1, -1), (1, 1)];
const BLACK_PAWN_ATTACKS: [TableMove; 2] = [(-1, -1), (-1, 1)];

#[rustfmt::skip]
const KNIGHTS_MOVES: [TableMove; 8] = [
    ( 2, -1), ( 2, 1),
    ( 1, -2), ( 1, 2),
    (-1, -2), (-1, 2),
    (-2, -1), (-2, 1),
];

// ------------- STUFF TO MAKE THE SLIDING PIECES WORK -------------

// Actually just the same as the King moves
#[rustfmt::skip]
const RAY_MOVES: [TableMove; 8] = [
    ( 1, -1), ( 1, 0), ( 1, 1),
    ( 0, -1),          ( 0, 1),
    (-1, -1), (-1, 0), (-1, 1),
];

// -------------                                       -------------

pub fn generate_lookup_tables() {
    println!("// This file was auto generated by src/table_generator.rs!");
    println!("// Do not modify this file unless you are pretty clear about what you are doing.");

    println!("use crate::board::{{Board, color::Color}};");
    println!("use super::utils::Direction;");

    // PAWN MOVES
    println!();

    println!("#[rustfmt::skip]");
    print!("pub const PAWN_PUSHES: [[u64; Board::SIZE]; Color::COUNT] = [[");
    let black_pawn_moves = generate_moves(&BLACK_PAWN_MOVE);
    for (index, bb) in black_pawn_moves.iter().enumerate() {
        if index % 8 == 0 {
            print!("\n\t");
        }

        print!("{:X}, ", bb);
    }
    print!("\n], [");
    let white_pawn_moves = generate_moves(&WHITE_PAWN_MOVE);
    for (index, bb) in white_pawn_moves.iter().enumerate() {
        if index % 8 == 0 {
            print!("\n\t");
        }

        print!("{:X}, ", bb);
    }
    println!("\n]];");

    // PAWN ATTACKS
    println!();

    println!("#[rustfmt::skip]");
    print!("pub const PAWN_ATTACKS: [[u64; Board::SIZE]; Color::COUNT] = [[");
    let black_pawn_moves = generate_moves(&BLACK_PAWN_ATTACKS);
    for (index, bb) in black_pawn_moves.iter().enumerate() {
        if index % 8 == 0 {
            print!("\n\t");
        }

        print!("{:X}, ", bb);
    }
    print!("\n], [");
    let white_pawn_moves = generate_moves(&WHITE_PAWN_ATTACKS);
    for (index, bb) in white_pawn_moves.iter().enumerate() {
        if index % 8 == 0 {
            print!("\n\t");
        }

        print!("{:X}, ", bb);
    }
    println!("\n]];");

    // KING MOVES
    println!();

    println!("#[rustfmt::skip]");
    print!("pub const KING_MOVES: [u64; Board::SIZE] = [");
    let king_moves = generate_moves(&KING_MOVES);
    for (index, bb) in king_moves.iter().enumerate() {
        if index % 8 == 0 {
            print!("\n\t");
        }

        print!("{:X}, ", bb);
    }
    println!("\n];");

    // KNIGHT MOVES
    println!();

    println!("#[rustfmt::skip]");
    print!("pub const KNIGHT_MOVES: [u64; Board::SIZE] = [");
    let knight_moves = generate_moves(&KNIGHTS_MOVES);
    for (index, bb) in knight_moves.iter().enumerate() {
        if index % 8 == 0 {
            print!("\n\t");
        }

        print!("{:X}, ", bb);
    }
    println!("\n];");

    // RAYS
    println!();

    let rays = generate_rays();

    println!("#[rustfmt::skip]");
    println!("pub const RAYS: [[u64; Direction::COUNT]; Board::SIZE] = [");
    for index in 0..Board::SIZE {
        print!("\t[ ");

        for ray in &rays[index] {
            print!("{:X}, ", ray);
        }

        println!("],");
    }
    println!("];");

    // BETWEEN LOOKUP
    println!();

    println!("#[rustfmt::skip]");
    print!("pub const BETWEEN_LOOKUP: [[u64; Board::SIZE]; Board::SIZE] = [");
    for from in 0..Board::SIZE {
        print!("\n\t[ ");

        let from = Square::index(from);
        for to in 0..Board::SIZE {
            let to = Square::index(to);
            let in_between = squares_between(from, to);
            print!("{:X}, ", in_between);
        }

        print!("],");
    }
    println!("\n];");

    // BETWEEN LOOKUP
    println!();

    println!("#[rustfmt::skip]");
    print!("pub const DIRECTION_LOOKUP: [[Direction; Board::SIZE]; Board::SIZE] = [");
    for from in 0..Board::SIZE {
        print!("\n\t[ ");

        let from = Square::index(from);
        for to in 0..Board::SIZE {
            let to = Square::index(to);
            let direction = get_direction(from, to);
            print!("Direction::{:?}, ", direction);
        }

        print!("],");
    }
    println!("\n];");
}

fn generate_moves(mask: &[TableMove]) -> [Bitboard; Board::SIZE] {
    let mut moves = [Bitboard::default(); Board::SIZE];

    for rank in 0..8 {
        for file in 0..8 {
            let from = Square::new(rank, file);

            for (d_rank, d_file) in mask {
                let rank = rank as i8 + d_rank;
                let file = file as i8 + d_file;
                if !inside_board(rank, file) {
                    continue;
                }

                let to = Square::new(rank as u8, file as u8);
                moves[from.index] |= to;
            }
        }
    }

    moves
}

fn generate_rays() -> [[Bitboard; Direction::COUNT]; Board::SIZE] {
    let mut rays = [[Bitboard::default(); Direction::COUNT]; Board::SIZE];

    for rank in 0..8 {
        for file in 0..8 {
            let from = Square::new(rank, file);

            for (ray_direction, (d_rank, d_file)) in RAY_MOVES.iter().enumerate() {
                let mut rank = rank as i8 + d_rank;
                let mut file = file as i8 + d_file;

                while inside_board(rank, file) {
                    let to = Square::new(rank as u8, file as u8);
                    rays[from.index][ray_direction] |= to;

                    rank += d_rank;
                    file += d_file;
                }
            }
        }
    }

    rays
}

fn squares_between(from: Square, to: Square) -> Bitboard {
    let ray_move = match from.get_direction(to) {
        Some(direction) => RAY_MOVES[direction.index()],
        None => return Bitboard::default(),
    };

    let d_rank = ray_move.0;
    let d_file = ray_move.1;

    let mut rank = from.rank() as i8 + d_rank;
    let mut file = from.file() as i8 + d_file;

    let mut result = Bitboard::default();
    loop {
        if !inside_board(rank, file) {
            break;
        }

        let delta_square = Square::new(rank as u8, file as u8);
        if delta_square == to {
            break;
        }

        result |= delta_square;

        rank += d_rank;
        file += d_file;
    }

    result
}

pub fn get_direction(from: Square, to: Square) -> Direction {
    let rank_cmp = to.rank().cmp(&from.rank());
    let file_cmp = to.file().cmp(&from.file());
    if rank_cmp.is_eq() && file_cmp.is_eq() {
        return Direction::None;
    }

    let rank_diff = to.rank() as i8 - from.rank() as i8;
    let file_diff = to.file() as i8 - from.file() as i8;
    let equal_delta = rank_diff.abs() == file_diff.abs();

    return match (rank_cmp, file_cmp, equal_delta) {
        (Ordering::Greater, Ordering::Less, true) => Direction::NorthWest,
        (Ordering::Greater, Ordering::Equal, false) => Direction::North,
        (Ordering::Greater, Ordering::Greater, true) => Direction::NorthEast,

        (Ordering::Equal, Ordering::Less, false) => Direction::West,
        (Ordering::Equal, Ordering::Greater, false) => Direction::East,

        (Ordering::Less, Ordering::Less, true) => Direction::SouthWest,
        (Ordering::Less, Ordering::Equal, false) => Direction::South,
        (Ordering::Less, Ordering::Greater, true) => Direction::SouthEast,

        _ => return Direction::None,
    };
}
