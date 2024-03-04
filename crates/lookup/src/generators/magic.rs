use std::u64;

use crate::{
    utils::{
        direction::Direction,
        utils::{random_64_few_bits, rank_file},
    },
    BOARD_SIZE,
};

const RANK_1: u64 = 0x00000000000000FF;
const RANK_8: u64 = 0xFF00000000000000;
const FILE_A: u64 = 0x0101010101010101;
const FILE_H: u64 = 0x8080808080808080;

pub fn generate_rook_masks(rays: &[[u64; 8]; BOARD_SIZE]) -> [u64; BOARD_SIZE] {
    let mut masks = [0; BOARD_SIZE];

    for from in 0..BOARD_SIZE {
        let (rank, file) = rank_file(from);
        let mut result = 0;

        result |= rays[from][Direction::North.index()];
        result |= rays[from][Direction::East.index()];
        result |= rays[from][Direction::South.index()];
        result |= rays[from][Direction::West.index()];

        if rank >= 1 {
            result &= !RANK_1;
        }
        if rank <= 6 {
            result &= !RANK_8;
        }

        if file >= 1 {
            result &= !FILE_A;
        }
        if file <= 6 {
            result &= !FILE_H;
        }

        masks[from] = result;
    }

    masks
}

pub fn generate_rook_mask_ones(masks: &[u64; BOARD_SIZE]) -> [u32; BOARD_SIZE] {
    let mut mask_ones = [0; BOARD_SIZE];

    for from in 0..BOARD_SIZE {
        let mask = masks[from];
        mask_ones[from] = mask.count_ones();
    }

    mask_ones
}

pub fn generate_rook_magics(
    rays: &[[u64; 8]; BOARD_SIZE],
    masks: &[u64; BOARD_SIZE],
    mask_ones: &[u32; BOARD_SIZE],
) -> [u64; BOARD_SIZE] {
    let mut magics = [0; BOARD_SIZE];

    for from in 0..BOARD_SIZE {
        let ones = mask_ones[from];
        let mask = masks[from];

        let result = find_magic(rays, from, mask, ones, false);
        if let Some(result) = result {
            magics[from] = result;
        } else {
            assert!(false, "Failed to find magic for rook");
        }
    }

    magics
}

pub fn generate_rook_attacks(
    rays: &[[u64; 8]; BOARD_SIZE],
    masks: &[u64; BOARD_SIZE],
    mask_ones: &[u32; BOARD_SIZE],
    magics: &[u64; BOARD_SIZE],
) -> [[u64; 4096]; 64] {
    let mut attacks = [[0; 4096]; 64];

    for from in 0..BOARD_SIZE {
        let ones = mask_ones[from];
        let magic = magics[from];
        let mask = masks[from];

        let permutations = 1 << ones;
        for index in 0..permutations {
            let blockers = permutate(index, ones, mask);
            let magic_index = get_magic_index(blockers, magic, ones);
            let rook_attacks = rook_attacks(rays, from, blockers);
            attacks[from][magic_index] = rook_attacks;
        }
    }

    attacks
}

pub fn generate_bishop_masks(rays: &[[u64; 8]; BOARD_SIZE]) -> [u64; BOARD_SIZE] {
    let mut masks = [0; BOARD_SIZE];

    for from in 0..BOARD_SIZE {
        let (rank, file) = rank_file(from);
        let mut result = 0;

        result |= rays[from][Direction::NorthEast.index()];
        result |= rays[from][Direction::NorthWest.index()];
        result |= rays[from][Direction::SouthEast.index()];
        result |= rays[from][Direction::SouthWest.index()];

        if rank >= 1 {
            result &= !RANK_1;
        }
        if rank <= 6 {
            result &= !RANK_8;
        }

        if file >= 1 {
            result &= !FILE_A;
        }
        if file <= 6 {
            result &= !FILE_H;
        }

        masks[from] = result;
    }

    masks
}

pub fn generate_bishop_mask_ones(masks: &[u64; BOARD_SIZE]) -> [u32; BOARD_SIZE] {
    let mut mask_ones = [0; BOARD_SIZE];

    for from in 0..BOARD_SIZE {
        let mask = masks[from];
        mask_ones[from] = mask.count_ones();
    }

    mask_ones
}

pub fn generate_bishop_magics(
    rays: &[[u64; 8]; BOARD_SIZE],
    masks: &[u64; BOARD_SIZE],
    mask_ones: &[u32; BOARD_SIZE],
) -> [u64; BOARD_SIZE] {
    let mut magics = [0; BOARD_SIZE];

    for from in 0..BOARD_SIZE {
        let ones = mask_ones[from];
        let mask = masks[from];

        let result = find_magic(rays, from, mask, ones, true);
        if let Some(result) = result {
            magics[from] = result;
        } else {
            assert!(false, "Failed to find magic for bishop");
        }
    }

    magics
}

pub fn generate_bishop_attacks(
    rays: &[[u64; 8]; BOARD_SIZE],
    masks: &[u64; BOARD_SIZE],
    mask_ones: &[u32; BOARD_SIZE],
    magics: &[u64; BOARD_SIZE],
) -> [[u64; 512]; 64] {
    let mut attacks = [[0; 512]; 64];

    for from in 0..BOARD_SIZE {
        let ones = mask_ones[from];
        let magic = magics[from];
        let mask = masks[from];

        let permutations = 1 << ones;
        for index in 0..permutations {
            let blockers = permutate(index, ones, mask);
            let magic_index = get_magic_index(blockers, magic, ones);
            let bishop_attacks = bishop_attacks(rays, from, blockers);
            attacks[from][magic_index] = bishop_attacks;
        }
    }

    attacks
}

fn find_magic(
    rays: &[[u64; 8]; BOARD_SIZE],
    from: usize,
    mask: u64,
    ones: u32,
    bishop: bool,
) -> Option<u64> {
    let mut permutations = [0; 4096];
    let mut attacks = [0; 4096];

    let permutation_count = 1 << ones;
    for index in 0..permutation_count {
        let permutation = permutate(index, ones, mask);
        permutations[index] = permutation;

        attacks[index] = if bishop {
            bishop_attacks(rays, from, permutation)
        } else {
            rook_attacks(rays, from, permutation)
        };
    }

    let mut seed = 4213371337;
    for _ in 0..100_000_000 {
        let result = random_64_few_bits(seed);
        seed = result.1;

        let magic = result.0;
        if !is_magic_canidate(mask, magic) {
            continue;
        }

        let mut used = [0; 4096];
        let mut failed = false;

        for index in 0..permutation_count {
            let permutation = permutations[index];
            let magic_index = get_magic_index(permutation, magic, ones);
            let magic_index = magic_index as usize;

            if used[magic_index] == 0 {
                used[magic_index] = attacks[index];
            } else if used[magic_index] != attacks[index] {
                failed = true;
                break;
            }
        }

        if !failed {
            return Some(magic);
        }
    }

    None
}

fn is_magic_canidate(mask: u64, magic: u64) -> bool {
    let candidate = mask.wrapping_mul(magic) & 0xFF00000000000000;
    candidate.count_ones() >= 6
}

fn get_magic_index(permutation: u64, magic: u64, ones: u32) -> usize {
    (permutation.wrapping_mul(magic) >> (64 - ones)) as usize
}

fn permutate(index: usize, bit_count: u32, mut mask: u64) -> u64 {
    let mut result = 0u64;

    for bit_index in 0..bit_count {
        let current_bit = mask.trailing_zeros() as u64;
        mask ^= 1 << current_bit;

        if (index & (1 << bit_index)) != 0 {
            result |= 1 << current_bit;
        }
    }

    result
}

fn rook_attacks(rays: &[[u64; 8]; BOARD_SIZE], from: usize, blockers: u64) -> u64 {
    let mut moves = get_ray_moves(rays, from, blockers, Direction::North, false);
    moves |= get_ray_moves(rays, from, blockers, Direction::East, false);
    moves |= get_ray_moves(rays, from, blockers, Direction::South, true);
    moves |= get_ray_moves(rays, from, blockers, Direction::West, true);
    moves
}

fn bishop_attacks(rays: &[[u64; 8]; BOARD_SIZE], from: usize, blockers: u64) -> u64 {
    let mut moves = get_ray_moves(rays, from, blockers, Direction::NorthEast, false);
    moves |= get_ray_moves(rays, from, blockers, Direction::SouthEast, true);
    moves |= get_ray_moves(rays, from, blockers, Direction::SouthWest, true);
    moves |= get_ray_moves(rays, from, blockers, Direction::NorthWest, false);
    moves
}

fn get_ray_moves(
    rays: &[[u64; 8]; BOARD_SIZE],
    from: usize,
    blockers: u64,
    direction: Direction,
    leading: bool,
) -> u64 {
    let mut moves = 0;

    moves |= rays[from][direction.index()];

    let blocking = moves & blockers;
    if blocking != 0 {
        let blocker_index = match leading {
            false => blocking.trailing_zeros() as usize,
            true => 63 - (blocking.leading_zeros() as usize),
        };

        moves &= !rays[blocker_index][direction.index()];
    }

    moves
}
