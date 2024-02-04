use std::cmp::Ordering;

use crate::{
    bitboard::{square::Square, Bitboard},
    board::{piece::Piece, Board},
};

use self::utils::Direction;

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

// ------------- STUFF FOR PeSTO's EVALUATION TABLES -------------

#[rustfmt::skip]
pub const MIDGAME_PAWN_TABLE: [isize; Board::SIZE] = [
      0,   0,   0,   0,   0,   0,  0,   0,
     98, 134,  61,  95,  68, 126, 34, -11,
     -6,   7,  26,  31,  65,  56, 25, -20,
    -14,  13,   6,  21,  23,  12, 17, -23,
    -27,  -2,  -5,  12,  17,   6, 10, -25,
    -26,  -4,  -4, -10,   3,   3, 33, -12,
    -35,  -1, -20, -23, -15,  24, 38, -22,
      0,   0,   0,   0,   0,   0,  0,   0,
];

#[rustfmt::skip]
pub const ENDGAME_PAWN_TABLE: [isize; Board::SIZE] = [
      0,   0,   0,   0,   0,   0,   0,   0,
    178, 173, 158, 134, 147, 132, 165, 187,
     94, 100,  85,  67,  56,  53,  82,  84,
     32,  24,  13,   5,  -2,   4,  17,  17,
     13,   9,  -3,  -7,  -7,  -8,   3,  -1,
      4,   7,  -6,   1,   0,  -5,  -1,  -8,
     13,   8,   8,  10,  13,   0,   2,  -7,
      0,   0,   0,   0,   0,   0,   0,   0,
];

#[rustfmt::skip]
pub const MIDGAME_KNIGHT_TABLE: [isize; Board::SIZE] = [
    -167, -89, -34, -49,  61, -97, -15, -107,
     -73, -41,  72,  36,  23,  62,   7,  -17,
     -47,  60,  37,  65,  84, 129,  73,   44,
      -9,  17,  19,  53,  37,  69,  18,   22,
     -13,   4,  16,  13,  28,  19,  21,   -8,
     -23,  -9,  12,  10,  19,  17,  25,  -16,
     -29, -53, -12,  -3,  -1,  18, -14,  -19,
    -105, -21, -58, -33, -17, -28, -19,  -23,
];

#[rustfmt::skip]
pub const ENDGAME_KNIGHT_TABLE: [isize; Board::SIZE] = [
    -58, -38, -13, -28, -31, -27, -63, -99,
    -25,  -8, -25,  -2,  -9, -25, -24, -52,
    -24, -20,  10,   9,  -1,  -9, -19, -41,
    -17,   3,  22,  22,  22,  11,   8, -18,
    -18,  -6,  16,  25,  16,  17,   4, -18,
    -23,  -3,  -1,  15,  10,  -3, -20, -22,
    -42, -20, -10,  -5,  -2, -20, -23, -44,
    -29, -51, -23, -15, -22, -18, -50, -64,
];

#[rustfmt::skip]
pub const MIDGAME_BISHOP_TABLE: [isize; Board::SIZE] = [
    -29,   4, -82, -37, -25, -42,   7,  -8,
    -26,  16, -18, -13,  30,  59,  18, -47,
    -16,  37,  43,  40,  35,  50,  37,  -2,
     -4,   5,  19,  50,  37,  37,   7,  -2,
     -6,  13,  13,  26,  34,  12,  10,   4,
      0,  15,  15,  15,  14,  27,  18,  10,
      4,  15,  16,   0,   7,  21,  33,   1,
    -33,  -3, -14, -21, -13, -12, -39, -21,
];

#[rustfmt::skip]
pub const ENDGAME_BISHOP_TABLE: [isize; Board::SIZE] = [
    -14, -21, -11,  -8, -7,  -9, -17, -24,
     -8,  -4,   7, -12, -3, -13,  -4, -14,
      2,  -8,   0,  -1, -2,   6,   0,   4,
     -3,   9,  12,   9, 14,  10,   3,   2,
     -6,   3,  13,  19,  7,  10,  -3,  -9,
    -12,  -3,   8,  10, 13,   3,  -7, -15,
    -14, -18,  -7,  -1,  4,  -9, -15, -27,
    -23,  -9, -23,  -5, -9, -16,  -5, -17,
];

#[rustfmt::skip]
pub const MIDGAME_ROOK_TABLE: [isize; Board::SIZE] = [
     32,  42,  32,  51, 63,  9,  31,  43,
     27,  32,  58,  62, 80, 67,  26,  44,
     -5,  19,  26,  36, 17, 45,  61,  16,
    -24, -11,   7,  26, 24, 35,  -8, -20,
    -36, -26, -12,  -1,  9, -7,   6, -23,
    -45, -25, -16, -17,  3,  0,  -5, -33,
    -44, -16, -20,  -9, -1, 11,  -6, -71,
    -19, -13,   1,  17, 16,  7, -37, -26,
];

#[rustfmt::skip]
pub const ENDGAME_ROOK_TABLE: [isize; Board::SIZE] = [
    13, 10, 18, 15, 12,  12,   8,   5,
    11, 13, 13, 11, -3,   3,   8,   3,
     7,  7,  7,  5,  4,  -3,  -5,  -3,
     4,  3, 13,  1,  2,   1,  -1,   2,
     3,  5,  8,  4, -5,  -6,  -8, -11,
    -4,  0, -5, -1, -7, -12,  -8, -16,
    -6, -6,  0,  2, -9,  -9, -11,  -3,
    -9,  2,  3, -1, -5, -13,   4, -20,
];

#[rustfmt::skip]
pub const MIDGAME_QUEEN_TABLE: [isize; Board::SIZE] = [
    -28,   0,  29,  12,  59,  44,  43,  45,
    -24, -39,  -5,   1, -16,  57,  28,  54,
    -13, -17,   7,   8,  29,  56,  47,  57,
    -27, -27, -16, -16,  -1,  17,  -2,   1,
     -9, -26,  -9, -10,  -2,  -4,   3,  -3,
    -14,   2, -11,  -2,  -5,   2,  14,   5,
    -35,  -8,  11,   2,   8,  15,  -3,   1,
     -1, -18,  -9,  10, -15, -25, -31, -50,
];

#[rustfmt::skip]
pub const ENDGAME_QUEEN_TABLE: [isize; Board::SIZE] = [
     -9,  22,  22,  27,  27,  19,  10,  20,
    -17,  20,  32,  41,  58,  25,  30,   0,
    -20,   6,   9,  49,  47,  35,  19,   9,
      3,  22,  24,  45,  57,  40,  57,  36,
    -18,  28,  19,  47,  31,  34,  39,  23,
    -16, -27,  15,   6,   9,  17,  10,   5,
    -22, -23, -30, -16, -16, -23, -36, -32,
    -33, -28, -22, -43,  -5, -32, -20, -41,
];

#[rustfmt::skip]
pub const MIDGAME_KING_TABLE: [isize; Board::SIZE] = [
    -65,  23,  16, -15, -56, -34,   2,  13,
     29,  -1, -20,  -7,  -8,  -4, -38, -29,
     -9,  24,   2, -16, -20,   6,  22, -22,
    -17, -20, -12, -27, -30, -25, -14, -36,
    -49,  -1, -27, -39, -46, -44, -33, -51,
    -14, -14, -22, -46, -44, -30, -15, -27,
      1,   7,  -8, -64, -43, -16,   9,   8,
    -15,  36,  12, -54,   8, -28,  24,  14,
];

#[rustfmt::skip]
pub const ENDGAME_KING_TABLE: [isize; Board::SIZE] = [
    -74, -35, -18, -18, -11,  15,   4, -17,
    -12,  17,  14,  17,  17,  38,  23,  11,
     10,  17,  23,  15,  20,  45,  44,  13,
     -8,  22,  24,  27,  26,  33,  26,   3,
    -18,  -4,  21,  24,  27,  23,   9, -11,
    -19,  -3,  11,  21,  23,  16,   7,  -9,
    -27, -11,   4,  13,  14,   4,  -5, -17,
    -53, -34, -21, -11, -28, -14, -24, -43
];

#[rustfmt::skip]
pub const MIDGAME_PIECE_VALUE: [isize; Piece::COUNT] = [82, 337, 365, 477, 1025, 0];

#[rustfmt::skip]
pub const ENDGAME_PIECE_VALUE: [isize; Piece::COUNT] = [94, 281, 297, 512, 936, 0];

#[rustfmt::skip]
pub const GAMEPHASE_INCREMENT: [isize; Piece::COUNT] = [ 0, 1, 1, 2, 4, 0 ];

// -------------                                     -------------

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

fn inside_board(rank: i8, file: i8) -> bool {
    let between_rank = rank >= Board::MIN_RANK as i8 && rank <= Board::MAX_RANK as i8;
    let between_file = file >= Board::MIN_FILE as i8 && file <= Board::MAX_FILE as i8;
    between_rank && between_file
}
