#![allow(dead_code)]

use std::cmp::Ordering;

// Defines a relative move in the format of (rank, file)
type Move = (i8, i8);

// ------------- ALL MOVES TO GENERATE THE TABLE -------------

#[rustfmt::skip]
const KING_MOVES: [Move; 8] = [
    ( 1, -1), ( 1, 0), ( 1, 1),
    ( 0, -1),          ( 0, 1),
    (-1, -1), (-1, 0), (-1, 1),
];

const WHITE_PAWN_MOVE: [Move; 1] = [(1, 0)];
const BLACK_PAWN_MOVE: [Move; 1] = [(-1, 0)];

const WHITE_PAWN_ATTACKS: [Move; 2] = [(1, -1), (1, 1)];
const BLACK_PAWN_ATTACKS: [Move; 2] = [(-1, -1), (-1, 1)];

#[rustfmt::skip]
const KNIGHTS_MOVES: [Move; 8] = [
    ( 2, -1), ( 2, 1),
    ( 1, -2), ( 1, 2),
    (-1, -2), (-1, 2),
    (-2, -1), (-2, 1),
];

// ------------- STUFF TO MAKE THE SLIDING PIECES WORK -------------

// Actually just the same as the King moves
#[rustfmt::skip]
const RAY_MOVES: [Move; 8] = [
    ( 1, -1), ( 1, 0), ( 1, 1),
    ( 0, -1),          ( 0, 1),
    (-1, -1), (-1, 0), (-1, 1),
];

const FILE_0: u64 = 0x101010101010101;
const FILE_7: u64 = 0x8080808080808080;

const RANK_0: u64 = 0xff;
const RANK_7: u64 = 0xff00000000000000;

// -------------                                       -------------

fn main() {
    println!("// This file was auto generated by src/table_generator.rs!");
    println!("// Do not modify this file unless you are pretty clear about what you are doing.");

    // PAWN MOVES
    println!();

    println!("#[rustfmt::skip]");
    print!("pub const PAWN_PUSHES: [[u64; 64]; 2] = [[");
    let black_pawn_moves = generate_moves(&BLACK_PAWN_MOVE);
    for (index, bits) in black_pawn_moves.iter().enumerate() {
        if index % 8 == 0 {
            print!("\n\t");
        }
        print!("0x{:x}, ", bits);
    }
    print!("\n], [");
    let white_pawn_moves = generate_moves(&WHITE_PAWN_MOVE);
    for (index, bits) in white_pawn_moves.iter().enumerate() {
        if index % 8 == 0 {
            print!("\n\t");
        }
        print!("0x{:x}, ", bits);
    }
    println!("\n]];");

    // PAWN ATTACKS
    println!();

    println!("#[rustfmt::skip]");
    print!("pub const PAWN_ATTACKS: [[u64; 64]; 2] = [[");
    let black_pawn_moves = generate_moves(&BLACK_PAWN_ATTACKS);
    for (index, bits) in black_pawn_moves.iter().enumerate() {
        if index % 8 == 0 {
            print!("\n\t");
        }
        print!("0x{:x}, ", bits);
    }
    print!("\n], [");
    let white_pawn_moves = generate_moves(&WHITE_PAWN_ATTACKS);
    for (index, bits) in white_pawn_moves.iter().enumerate() {
        if index % 8 == 0 {
            print!("\n\t");
        }
        print!("0x{:x}, ", bits);
    }
    println!("\n]];");

    // KING MOVES
    println!();

    println!("#[rustfmt::skip]");
    print!("pub const KING_MOVES: [u64; 64] = [");
    let king_moves = generate_moves(&KING_MOVES);
    for (index, bits) in king_moves.iter().enumerate() {
        if index % 8 == 0 {
            print!("\n\t");
        }
        print!("0x{:x}, ", bits);
    }
    println!("\n];");

    // KNIGHT MOVES
    println!();

    println!("#[rustfmt::skip]");
    print!("pub const KNIGHT_MOVES: [u64; 64] = [");
    let knight_moves = generate_moves(&KNIGHTS_MOVES);
    for (index, bits) in knight_moves.iter().enumerate() {
        if index % 8 == 0 {
            print!("\n\t");
        }
        print!("0x{:x}, ", bits);
    }
    println!("\n];");

    // RAYS
    println!();

    let rays = generate_rays();

    println!("#[rustfmt::skip]");
    println!("pub const RAYS: [[u64; 8]; 64] = [");
    for index in 0..64 {
        print!("\t[ ");
        for ray in &rays[index] {
            print!("0x{:x}, ", ray);
        }
        println!("],");
    }
    println!("];");

    // ROOK RAYS
    println!();

    println!("#[rustfmt::skip]");
    print!("pub const COMBINED_ROOK_RAYS: [u64; 64] = [");
    for index in 0..64 {
        if index % 8 == 0 {
            print!("\n\t");
        }

        let combined_ray = rays[index][1] | rays[index][3] | rays[index][4] | rays[index][6];
        print!("0x{:x}, ", combined_ray);
    }
    println!("\n];");

    // BISHOP RAYS
    println!();

    println!("#[rustfmt::skip]");
    print!("pub const COMBINED_BISHOP_RAYS: [u64; 64] = [");
    for index in 0..64 {
        if index % 8 == 0 {
            print!("\n\t");
        }

        let combined_ray = rays[index][0] | rays[index][2] | rays[index][5] | rays[index][7];
        print!("0x{:x}, ", combined_ray);
    }
    println!("\n];");

    // BETWEEN LOOKUP
    println!();

    println!("#[rustfmt::skip]");
    print!("pub const BETWEEN_LOOKUP: [[u64; 64]; 64] = [");
    for from in 0..64 {
        print!("\n\t[ ");
        for to in 0..64 {
            let in_between = in_between(from, to);
            print!("0x{:x}, ", in_between);
        }
        print!("],");
    }
    println!("\n];");
}

fn generate_moves(mask: &[Move]) -> [u64; 64] {
    let mut moves = [0u64; 64];

    for rank in 0..8 {
        for file in 0..8 {
            let index = ((8 * rank) + file) as usize;

            for (d_rank, d_file) in mask {
                let rank = rank as i8 + d_rank;
                let file = file as i8 + d_file;
                if !inside_board(rank, file) {
                    continue;
                }

                let d_index = ((8 * rank) + file) as usize;
                moves[index] |= 1u64 << d_index;
            }
        }
    }

    moves
}

fn generate_rays() -> [[u64; 8]; 64] {
    let mut rays = [[0u64; 8]; 64];

    for rank in 0..8 {
        for file in 0..8 {
            let index = ((8 * rank) + file) as usize;

            for (ray_index, (d_rank, d_file)) in RAY_MOVES.iter().enumerate() {
                let mut rank = rank as i8 + d_rank;
                let mut file = file as i8 + d_file;

                while inside_board(rank, file) {
                    let d_index = ((8 * rank) + file) as usize;
                    rays[index][ray_index] |= 1u64 << d_index;

                    rank += d_rank;
                    file += d_file;
                }
            }
        }
    }

    rays
}

fn in_between(from_index: usize, to_index: usize) -> u64 {
    let direction = match get_direction_index(from_index, to_index) {
        Some(index) => RAY_MOVES[index],
        None => return 0,
    };

    let from_rank = from_index / 8;
    let d_rank = direction.0;
    let from_file = from_index % 8;
    let d_file = direction.1;

    let mut rank = from_rank as i8 + d_rank;
    let mut file = from_file as i8 + d_file;

    let mut result = 0u64;
    loop {
        let d_index = ((8 * rank) + file) as usize;
        if to_index == d_index || !inside_board(rank, file) {
            break;
        }

        result |= 1u64 << d_index;

        rank += d_rank;
        file += d_file;
    }

    result
}

pub fn get_direction_index(from_index: usize, to_index: usize) -> Option<usize> {
    let to_rank = to_index / 8;
    let to_file = to_index % 8;
    let from_rank = from_index / 8;
    let from_file = from_index % 8;

    let rank_cmp = to_rank.cmp(&from_rank);
    let file_cmp = to_file.cmp(&from_file);
    if rank_cmp.is_eq() && file_cmp.is_eq() {
        return None;
    }

    let rank_diff = to_rank as i8 - from_rank as i8;
    let file_diff = to_file as i8 - from_file as i8;
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

fn inside_board(rank: i8, file: i8) -> bool {
    (rank >= 0 && rank <= 7) && (file >= 0 && file <= 7)
}
