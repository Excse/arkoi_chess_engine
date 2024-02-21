use const_for::*;
use const_random::const_random;

use crate::{direction::Direction, generic::RAYS, utils::{rank_file, random_64}, BOARD_SIZE};

pub(crate) const RANK_1: u64 = 0x00000000000000FF;
pub(crate) const RANK_8: u64 = 0xFF00000000000000;
pub(crate) const FILE_A: u64 = 0x0101010101010101;
pub(crate) const FILE_H: u64 = 0x8080808080808080;

#[rustfmt::skip]
pub const ROOK_MASKS: [u64; BOARD_SIZE] = {
    let mut masks = [0; BOARD_SIZE];

    const_for!(from in 0..BOARD_SIZE => {
        let (rank, file) = rank_file(from);
        let mut result = 0;

        result |= RAYS[from][Direction::North.index()];
        result |= RAYS[from][Direction::East.index()];
        result |= RAYS[from][Direction::South.index()];
        result |= RAYS[from][Direction::West.index()];

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
    });

    masks
};

#[rustfmt::skip]
pub const ROOK_MASK_ONES: [u32; BOARD_SIZE] = {
    let mut mask_ones = [0; BOARD_SIZE];

    const_for!(from in 0..BOARD_SIZE => {
        let mask = ROOK_MASKS[from];
        mask_ones[from] = mask.count_ones();
    });

    mask_ones
};

#[rustfmt::skip]
pub const ROOK_MAGICS: [u64; BOARD_SIZE] = {
    let mut magics = [0; BOARD_SIZE];

    const_for!(from in 0..BOARD_SIZE => {
        let ones = ROOK_MASK_ONES[from];
        let mask = ROOK_MASKS[from];

        let result = find_magic(from, mask, ones, false);
        if let Some(result) = result {
            magics[from] = result;
        } else {
            assert!(false, "Failed to find magic for rook");
        }
    });

    magics
};

#[rustfmt::skip]
pub const ROOK_ATTACKS: [[u64; 4096]; 64] = {
    let mut attacks = [[0; 4096]; 64];

    const_for!(from in 0..BOARD_SIZE => {
        let ones = ROOK_MASK_ONES[from];
        let magic = ROOK_MAGICS[from];
        let mask = ROOK_MASKS[from];

        let permutations = 1 << ones;
        const_for!(index in 0..permutations => {
            let blockers = permutate(index, ones, mask);
            let magic_index = get_magic_index(blockers, magic, ones);
            let rook_attacks = rook_attacks(from, blockers);
            attacks[from][magic_index] = rook_attacks;
        });
    });

    attacks
};

#[rustfmt::skip]
pub const BISHOP_MASKS: [u64; BOARD_SIZE] = {
    let mut masks = [0; BOARD_SIZE];

    const_for!(from in 0..BOARD_SIZE => {
        let (rank, file) = rank_file(from);
        let mut result = 0;

        result |= RAYS[from][Direction::NorthEast.index()];
        result |= RAYS[from][Direction::NorthWest.index()];
        result |= RAYS[from][Direction::SouthEast.index()];
        result |= RAYS[from][Direction::SouthWest.index()];

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
    });

    masks
};

#[rustfmt::skip]
pub const BISHOP_MASK_ONES: [u32; BOARD_SIZE] = {
    let mut mask_ones = [0; BOARD_SIZE];

    const_for!(from in 0..BOARD_SIZE => {
        let mask = BISHOP_MASKS[from];
        mask_ones[from] = mask.count_ones();
    });

    mask_ones
};

#[rustfmt::skip]
pub const BISHOP_MAGICS: [u64; BOARD_SIZE] = {
    let mut magics = [0; BOARD_SIZE];

    const_for!(from in 0..BOARD_SIZE => {
        let ones = BISHOP_MASK_ONES[from];
        let mask = BISHOP_MASKS[from];

        let result = find_magic(from, mask, ones, true);
        if let Some(result) = result {
            magics[from] = result;
        } else {
            assert!(false, "Failed to find magic for bishop");
        }
    });

    magics
};

#[rustfmt::skip]
pub const BISHOP_ATTACKS: [[u64; 512]; 64] = {
    let mut attacks = [[0; 512]; 64];

    const_for!(from in 0..BOARD_SIZE => {
        let ones = BISHOP_MASK_ONES[from];
        let magic = BISHOP_MAGICS[from];
        let mask = BISHOP_MASKS[from];

        let permutations = 1 << ones;
        const_for!(index in 0..permutations => {
            let blockers = permutate(index, ones, mask);
            let magic_index = get_magic_index(blockers, magic, ones);
            let bishop_attacks = bishop_attacks(from, blockers);
            attacks[from][magic_index] = bishop_attacks;
        });
    });

    attacks
};

const fn find_magic(from: usize, mask: u64, ones: u32, bishop: bool) -> Option<u64> {
    let mut permutations = [0; 4096];
    let mut attacks = [0; 4096];

    let permutation_count = 1 << ones;
    const_for!(index in 0..permutation_count => {
        let permutation = permutate(index, ones, mask);
        permutations[index] = permutation;

        attacks[index] = if bishop {
            bishop_attacks(from, permutation)
        } else {
            rook_attacks(from, permutation)
        };
    });

    let mut seed = const_random!(u32);
    const_for!(_ in 0..100_000_000 => {
        let result = random_64(seed);
        seed = result.1;

        let magic = result.0;
        if !is_magic_canidate(mask, magic) {
            continue;
        }

        let mut used = [0; 4096];
        let mut failed = false;

        const_for!(index in 0..permutation_count => {
            let permutation = permutations[index];
            let magic_index = get_magic_index(permutation, magic, ones);
            let magic_index = magic_index as usize;

            if used[magic_index] == 0 {
                used[magic_index] = attacks[index];
            } else if used[magic_index] != attacks[index] {
                failed = true;
                break;
            }
        });

        if !failed {
            return Some(magic);
        }
    });

    None
}

const fn is_magic_canidate(mask: u64, magic: u64) -> bool {
    let candidate = mask.wrapping_mul(magic) & 0xFF00000000000000;
    candidate.count_ones() >= 6
}

const fn get_magic_index(permutation: u64, magic: u64, ones: u32) -> usize {
    (permutation.wrapping_mul(magic) >> (64 - ones)) as usize
}

const fn permutate(index: usize, bit_count: u32, mut mask: u64) -> u64 {
    let mut result = 0u64;

    const_for!(bit_index in 0..bit_count => {
        let current_bit = mask.trailing_zeros() as u64;
        mask ^= 1 << current_bit;

        if (index & (1 << bit_index)) != 0 {
            result |= current_bit;
        }
    });

    result
}

const fn rook_attacks(from: usize, blockers: u64) -> u64 {
    let mut moves = get_ray_moves(from, blockers, Direction::North, false);
    moves |= get_ray_moves(from, blockers, Direction::East, false);
    moves |= get_ray_moves(from, blockers, Direction::South, true);
    moves |= get_ray_moves(from, blockers, Direction::West, true);
    moves
}

const fn bishop_attacks(from: usize, blockers: u64) -> u64 {
    let mut moves = get_ray_moves(from, blockers, Direction::NorthEast, false);
    moves |= get_ray_moves(from, blockers, Direction::SouthEast, true);
    moves |= get_ray_moves(from, blockers, Direction::SouthWest, true);
    moves |= get_ray_moves(from, blockers, Direction::NorthWest, false);
    moves
}

const fn get_ray_moves(from: usize, blockers: u64, direction: Direction, leading: bool) -> u64 {
    let mut moves = 0;

    moves |= RAYS[from][direction.index()];

    let blocking = moves & blockers;
    if blocking != 0 {
        let blocker_index = match leading {
            false => blocking.trailing_zeros() as usize,
            true => 63 - (blocking.leading_zeros() as usize),
        };

        moves &= !RAYS[blocker_index][direction.index()];
    }

    moves
}
