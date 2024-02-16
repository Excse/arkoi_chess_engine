use std::fmt::Write;

use rand::RngCore;

use crate::{
    bitboard::{constants::*, square::Square, Bitboard},
    board::Board,
};

use super::{utils::Direction, Rays};

type Result<T> = std::result::Result<T, std::fmt::Error>;

type Magics = [u64; Board::SIZE];
type Masks = [Bitboard; Board::SIZE];
type MaskOnes = [usize; Board::SIZE];

type RookAttacks = [[Bitboard; 4096]; Board::SIZE];
type BishopAttacks = [[Bitboard; 512]; Board::SIZE];

pub fn generate_magic(writer: &mut impl Write, rays: &Rays) -> Result<()> {
    let rook_masks = generate_rook_masks(writer, rays)?;
    let rook_mask_ones = generate_rook_mask_ones(writer, rays)?;
    let rook_magics = generate_rook_magics(writer, &rook_masks, &rook_mask_ones)?;
    generate_rook_attacks(writer, &rook_magics, &rook_masks, &rook_mask_ones)?;

    let bishop_masks = generate_bishop_masks(writer, rays)?;
    let bishop_mask_ones = generate_bishop_mask_ones(writer, rays)?;
    let bishop_magics = generate_bishop_magics(writer, &bishop_masks, &bishop_mask_ones)?;
    generate_bishop_attacks(writer, &bishop_magics, &bishop_masks, &bishop_mask_ones)?;

    Ok(())
}

pub fn generate_rook_magics(
    dest: &mut impl Write,
    masks: &Masks,
    ones: &MaskOnes,
) -> Result<Magics> {
    let mut magics = [0u64; Board::SIZE];
    for from_index in 0..Board::SIZE {
        let from = Square::from_index(from_index as u8);
        let magic = match find_magic(from, masks, ones, false) {
            Some(magic) => magic,
            _ => panic!("Could not find magic for {}", from),
        };

        magics[from_index] = magic;
    }

    writeln!(dest)?;
    writeln!(dest, "#[rustfmt::skip]")?;
    write!(dest, "pub const ROOK_MAGICS: [u64; Board::SIZE] = [")?;
    for (index, magic) in magics.iter().enumerate() {
        if index % 8 == 0 {
            write!(dest, "\n\t")?;
        }

        write!(dest, "0x{:X}, ", magic)?;
    }
    writeln!(dest, "\n];")?;

    Ok(magics)
}

pub fn generate_bishop_magics(
    dest: &mut impl Write,
    masks: &Masks,
    ones: &MaskOnes,
) -> Result<Magics> {
    let mut magics = [0u64; Board::SIZE];
    for from_index in 0..Board::SIZE {
        let from = Square::from_index(from_index as u8);
        let magic = match find_magic(from, masks, ones, true) {
            Some(magic) => magic,
            _ => panic!("Could not find magic for {}", from),
        };

        magics[from_index] = magic;
    }

    writeln!(dest)?;
    writeln!(dest, "#[rustfmt::skip]")?;
    write!(dest, "pub const BISHOP_MAGICS: [u64; Board::SIZE] = [")?;
    for (index, magic) in magics.iter().enumerate() {
        if index % 8 == 0 {
            write!(dest, "\n\t")?;
        }

        write!(dest, "0x{:X}, ", magic)?;
    }
    writeln!(dest, "\n];")?;

    Ok(magics)
}

pub fn generate_rook_attacks(
    dest: &mut impl Write,
    magics: &Magics,
    masks: &Masks,
    ones: &MaskOnes,
) -> Result<RookAttacks> {
    let mut attacks = [[Bitboard::default(); 4096]; Board::SIZE];

    for square_index in 0..Board::SIZE {
        let square = Square::from_index(square_index as u8);
        let mask = masks[square_index];
        let ones = ones[square_index];
        let magic = magics[square_index];

        let permutations = 1 << ones;
        for index in 0..permutations {
            let blockers = permutate(index, ones, mask);
            let magic_index = blockers.get_magic_index(magic, ones);
            let rook_attacks = rook_attacks(square, blockers);
            attacks[square_index][magic_index] = rook_attacks;
        }
    }

    writeln!(dest)?;
    writeln!(dest, "#[rustfmt::skip]")?;
    write!(
        dest,
        "pub const ROOK_ATTACKS: [[u64; 4096]; Board::SIZE] = ["
    )?;
    for from_index in 0..Board::SIZE {
        write!(dest, "\n\t[ ")?;

        for to in 0..4096 {
            let rook_attacks = attacks[from_index][to];
            write!(dest, "0x{:X}, ", rook_attacks)?;
        }

        write!(dest, "],")?;
    }
    writeln!(dest, "\n];")?;

    Ok(attacks)
}

pub fn generate_rook_masks(dest: &mut impl Write, rays: &Rays) -> Result<Masks> {
    let mut rook_masks = [Bitboard::default(); Board::SIZE];
    for from_index in 0..Board::SIZE {
        let from = Square::from_index(from_index as u8);
        let mask = get_rook_mask(from, &rays);
        rook_masks[from_index] = mask;
    }

    writeln!(dest)?;
    writeln!(dest, "#[rustfmt::skip]")?;
    write!(dest, "pub const ROOK_MASKS: [u64; Board::SIZE] = [")?;
    for (index, mask) in rook_masks.iter().enumerate() {
        if index % 8 == 0 {
            write!(dest, "\n\t")?;
        }

        write!(dest, "0x{:X}, ", mask)?;
    }
    writeln!(dest, "\n];")?;

    Ok(rook_masks)
}

pub fn generate_rook_mask_ones(dest: &mut impl Write, rays: &Rays) -> Result<MaskOnes> {
    let mut rook_mask_ones = [0usize; Board::SIZE];
    for from_index in 0..Board::SIZE {
        let from = Square::from_index(from_index as u8);
        let mask = get_rook_mask(from, &rays);
        let ones = mask.count_ones();
        rook_mask_ones[from_index] = ones;
    }

    writeln!(dest)?;
    writeln!(dest, "#[rustfmt::skip]")?;
    write!(dest, "pub const ROOK_MASK_ONES: [usize; Board::SIZE] = [")?;
    for (index, ones) in rook_mask_ones.iter().enumerate() {
        if index % 8 == 0 {
            write!(dest, "\n\t")?;
        }

        write!(dest, "{}, ", ones)?;
    }
    writeln!(dest, "\n];")?;

    Ok(rook_mask_ones)
}

pub fn generate_bishop_attacks(
    dest: &mut impl Write,
    magics: &Magics,
    masks: &Masks,
    ones: &MaskOnes,
) -> Result<BishopAttacks> {
    let mut attacks = [[Bitboard::default(); 512]; Board::SIZE];

    for square_index in 0..Board::SIZE {
        let square = Square::from_index(square_index as u8);
        let mask = masks[square_index];
        let ones = ones[square_index];
        let magic = magics[square_index];

        let permutations = 1 << ones;
        for index in 0..permutations {
            let blockers = permutate(index, ones, mask);
            let magic_index = blockers.get_magic_index(magic, ones);
            let bishop_attacks = bishop_attacks(square, blockers);
            attacks[square_index][magic_index] = bishop_attacks;
        }
    }

    writeln!(dest)?;
    writeln!(dest, "#[rustfmt::skip]")?;
    write!(
        dest,
        "pub const BISHOP_ATTACKS: [[u64; 512]; Board::SIZE] = ["
    )?;
    for from_index in 0..Board::SIZE {
        write!(dest, "\n\t[ ")?;

        for to in 0..512 {
            let bishop_attacks = attacks[from_index][to];
            write!(dest, "0x{:X}, ", bishop_attacks)?;
        }

        write!(dest, "],")?;
    }
    writeln!(dest, "\n];")?;

    Ok(attacks)
}

pub fn generate_bishop_masks(dest: &mut impl Write, rays: &Rays) -> Result<Masks> {
    let mut bishop_masks = [Bitboard::default(); Board::SIZE];
    for from_index in 0..Board::SIZE {
        let from = Square::from_index(from_index as u8);
        let mask = get_bishop_mask(from, &rays);
        bishop_masks[from_index] = mask;
    }

    writeln!(dest)?;
    writeln!(dest, "#[rustfmt::skip]")?;
    write!(dest, "pub const BISHOP_MASKS: [u64; Board::SIZE] = [")?;
    for (index, mask) in bishop_masks.iter().enumerate() {
        if index % 8 == 0 {
            write!(dest, "\n\t")?;
        }

        write!(dest, "0x{:X}, ", mask)?;
    }
    writeln!(dest, "\n];")?;

    Ok(bishop_masks)
}

pub fn generate_bishop_mask_ones(dest: &mut impl Write, rays: &Rays) -> Result<MaskOnes> {
    let mut bishop_mask_ones = [0usize; Board::SIZE];
    for from_index in 0..Board::SIZE {
        let from = Square::from_index(from_index as u8);
        let mask = get_bishop_mask(from, &rays);
        let ones = mask.count_ones();
        bishop_mask_ones[from_index] = ones;
    }

    writeln!(dest)?;
    writeln!(dest, "#[rustfmt::skip]")?;
    write!(dest, "pub const BISHOP_MASK_ONES: [usize; Board::SIZE] = [")?;
    for (index, ones) in bishop_mask_ones.iter().enumerate() {
        if index % 8 == 0 {
            write!(dest, "\n\t")?;
        }

        write!(dest, "{}, ", ones)?;
    }
    writeln!(dest, "\n];")?;

    Ok(bishop_mask_ones)
}

fn random_u64_few_bits() -> u64 {
    let mut rand = rand::thread_rng();
    rand.next_u64() & rand.next_u64() & rand.next_u64()
}

fn permutate(index: usize, bit_count: usize, mut mask: Bitboard) -> Bitboard {
    let mut result = Bitboard::default();

    for bit_index in 0..bit_count {
        let current_bit = mask.pop_trailing();

        if (index & (1 << bit_index)) != 0 {
            result |= current_bit;
        }
    }

    return result;
}

fn find_magic(square: Square, masks: &Masks, ones: &MaskOnes, bishop: bool) -> Option<u64> {
    let square_index = usize::from(square);
    let mask = masks[square_index];
    let ones = ones[square_index];

    let mut permutations = [Bitboard::default(); 4096];
    let mut attacks = [Bitboard::default(); 4096];

    let permutation_count = 1 << ones;
    for index in 0..permutation_count {
        let permutation = permutate(index, ones, mask);
        permutations[index] = permutation;

        attacks[index] = if bishop {
            bishop_attacks(square, permutation)
        } else {
            rook_attacks(square, permutation)
        };
    }

    for _ in 0..100_000_000 {
        let magic = random_u64_few_bits();
        if !mask.is_magic_canidate(magic) {
            continue;
        }

        let mut used = [Bitboard::default(); 4096];
        let mut failed = false;

        for index in 0..permutation_count {
            let permutation = permutations[index];
            let magic_index = permutation.get_magic_index(magic, ones);
            let magic_index = magic_index as usize;

            if used[magic_index].is_empty() {
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

fn rook_attacks(from: Square, blockers: Bitboard) -> Bitboard {
    let mut moves = get_ray_moves(from, blockers, Direction::North, false);
    moves |= get_ray_moves(from, blockers, Direction::East, false);
    moves |= get_ray_moves(from, blockers, Direction::South, true);
    moves |= get_ray_moves(from, blockers, Direction::West, true);
    moves
}

fn bishop_attacks(from: Square, blockers: Bitboard) -> Bitboard {
    let mut moves = get_ray_moves(from, blockers, Direction::NorthEast, false);
    moves |= get_ray_moves(from, blockers, Direction::SouthEast, true);
    moves |= get_ray_moves(from, blockers, Direction::SouthWest, true);
    moves |= get_ray_moves(from, blockers, Direction::NorthWest, false);
    moves
}

fn get_ray_moves(
    from: Square,
    blockers: Bitboard,
    direction: Direction,
    leading: bool,
) -> Bitboard {
    let mut moves = Bitboard::default();

    let ray = from.get_ray(direction);
    moves |= ray;

    let blocking = ray & blockers;
    if !blocking.is_empty() {
        let blocker_index = match leading {
            false => blocking.get_trailing_index(),
            true => blocking.get_leading_index(),
        };

        let blocker = Square::from_index(blocker_index);
        moves &= !blocker.get_ray(direction);
    }

    moves
}

fn get_rook_mask(from: Square, rays: &[[Bitboard; Direction::COUNT]; Board::SIZE]) -> Bitboard {
    let mut result = Bitboard::default();

    let from_index = usize::from(from);
    result |= rays[from_index][Direction::North.index()];
    result |= rays[from_index][Direction::East.index()];
    result |= rays[from_index][Direction::South.index()];
    result |= rays[from_index][Direction::West.index()];

    let rank = from.rank();
    if rank >= 1 {
        result &= !RANK_1;
    }
    if rank <= 6 {
        result &= !RANK_8;
    }

    let file = from.file();
    if file >= 1 {
        result &= !FILE_A;
    }
    if file <= 6 {
        result &= !FILE_H;
    }

    result
}

fn get_bishop_mask(from: Square, rays: &[[Bitboard; Direction::COUNT]; Board::SIZE]) -> Bitboard {
    let mut result = Bitboard::default();

    let from_index = usize::from(from);
    result |= rays[from_index][Direction::NorthEast.index()];
    result |= rays[from_index][Direction::NorthWest.index()];
    result |= rays[from_index][Direction::SouthEast.index()];
    result |= rays[from_index][Direction::SouthWest.index()];

    let rank = from.rank();
    if rank >= 1 {
        result &= !RANK_1;
    }
    if rank <= 6 {
        result &= !RANK_8;
    }

    let file = from.file();
    if file >= 1 {
        result &= !FILE_A;
    }
    if file <= 6 {
        result &= !FILE_H;
    }

    result
}
