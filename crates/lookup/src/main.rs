#![allow(dead_code)]

use std::{
    error::Error,
    fmt::{UpperHex, Write},
    path::Path,
};

use generators::moves::generate_pawn_attacks;

use crate::generators::{
    generic::{generate_adjacent_files, generate_between, generate_lines, generate_rays},
    magic::{
        generate_bishop_attacks, generate_bishop_magics, generate_bishop_mask_ones,
        generate_bishop_masks, generate_rook_attacks, generate_rook_magics,
        generate_rook_mask_ones, generate_rook_masks,
    },
    moves::{generate_king_moves, generate_knight_moves, generate_pawn_pushes},
};

pub(crate) const BOARD_SIZE: usize = 64;
pub(crate) const COLOR_COUNT: usize = 2;

pub mod generators;
pub mod utils;

fn main() -> Result<(), Box<dyn Error>> {
    let mut output = String::new();

    writeln!(
        output,
        "// This file was generated automatically. Do not change stuff here."
    )?;
    writeln!(
        output,
        "// Instead run the binary ourself if you changed something at the generators."
    )?;

    let king_moves = generate_king_moves();
    write_one_dimensional("KING_MOVES", king_moves.to_vec(), &mut output)?;

    let knight_moves = generate_knight_moves();
    write_one_dimensional("KNIGHT_MOVES", knight_moves.to_vec(), &mut output)?;

    let pawn_pushes = generate_pawn_pushes();
    let pawn_pushes_as_vec = pawn_pushes
        .iter()
        .map(|row| row.iter().cloned().collect::<Vec<u64>>())
        .collect::<Vec<Vec<u64>>>();
    write_two_dimensional("PAWN_PUSHES", pawn_pushes_as_vec, &mut output)?;

    let pawn_attacks = generate_pawn_attacks();
    let pawn_attacks_as_vec = pawn_attacks
        .iter()
        .map(|row| row.iter().cloned().collect::<Vec<u64>>())
        .collect::<Vec<Vec<u64>>>();
    write_two_dimensional("PAWN_ATTACKS", pawn_attacks_as_vec, &mut output)?;

    let between = generate_between();
    let between_as_vec = between
        .iter()
        .map(|row| row.iter().cloned().collect::<Vec<u64>>())
        .collect::<Vec<Vec<u64>>>();
    write_two_dimensional("BETWEEN", between_as_vec, &mut output)?;

    let lines = generate_lines();
    let lines_as_vec = lines
        .iter()
        .map(|row| row.iter().cloned().collect::<Vec<u64>>())
        .collect::<Vec<Vec<u64>>>();
    write_two_dimensional("LINES", lines_as_vec, &mut output)?;

    let adjacent_files = generate_adjacent_files();
    let adjacent_files_as_vec = adjacent_files.iter().cloned().collect::<Vec<u64>>();
    write_one_dimensional("ADJACENT_FILES", adjacent_files_as_vec, &mut output)?;

    let rays = generate_rays();
    let rays_as_vec = rays
        .iter()
        .map(|row| row.iter().cloned().collect::<Vec<u64>>())
        .collect::<Vec<Vec<u64>>>();
    write_two_dimensional("RAYS", rays_as_vec, &mut output)?;

    let r_masks = generate_rook_masks(&rays);
    write_one_dimensional("ROOK_MASKS", r_masks.to_vec(), &mut output)?;

    let b_masks = generate_bishop_masks(&rays);
    write_one_dimensional("BISHOP_MASKS", b_masks.to_vec(), &mut output)?;

    let r_mask_ones = generate_rook_mask_ones(&r_masks);
    write_one_dimensional("ROOK_MASK_ONES", r_mask_ones.to_vec(), &mut output)?;

    let b_mask_ones = generate_bishop_mask_ones(&b_masks);
    write_one_dimensional("BISHOP_MASK_ONES", b_mask_ones.to_vec(), &mut output)?;

    let r_magics = generate_rook_magics(&rays, &r_masks, &r_mask_ones);
    write_one_dimensional("ROOK_MAGICS", r_magics.to_vec(), &mut output)?;

    let b_magics = generate_bishop_magics(&rays, &b_masks, &b_mask_ones);
    write_one_dimensional("BISHOP_MAGICS", b_magics.to_vec(), &mut output)?;

    let r_magic_attacks = generate_rook_attacks(&rays, &r_masks, &r_mask_ones, &r_magics);
    let r_magic_attacks_as_vec = r_magic_attacks
        .iter()
        .map(|row| row.iter().cloned().collect::<Vec<u64>>())
        .collect::<Vec<Vec<u64>>>();
    write_two_dimensional("ROOK_MAGIC_ATTACKS", r_magic_attacks_as_vec, &mut output)?;

    let b_magic_attacks = generate_bishop_attacks(&rays, &b_masks, &b_mask_ones, &b_magics);
    let b_magic_attacks_as_vec = b_magic_attacks
        .iter()
        .map(|row| row.iter().cloned().collect::<Vec<u64>>())
        .collect::<Vec<Vec<u64>>>();
    write_two_dimensional("BISHOP_MAGIC_ATTACKS", b_magic_attacks_as_vec, &mut output)?;

    let path = Path::new("./src/tables/generated.rs");
    std::fs::write(path, output)?;

    Ok(())
}

fn write_one_dimensional<T: UpperHex, W: Write>(
    name: &str,
    input: Vec<T>,
    write: &mut W,
) -> Result<(), Box<dyn Error>> {
    write!(
        write,
        "pub const {}: [{}; {}] = [",
        name,
        std::any::type_name::<T>(),
        input.len(),
    )?;
    for (index, item) in input.iter().enumerate() {
        if index % 8 == 0 {
            write!(write, "\n\t")?;
        }

        write!(write, "0x{:X}, ", item)?;
    }
    writeln!(write, "\n];\n")?;

    Ok(())
}

fn write_two_dimensional<T: UpperHex, W: Write>(
    name: &str,
    input: Vec<Vec<T>>,
    write: &mut W,
) -> Result<(), Box<dyn Error>> {
    write!(
        write,
        "pub const {}: [[{}; {}]; {}] = [",
        name,
        std::any::type_name::<T>(),
        input[0].len(),
        input.len(),
    )?;
    for inner in input {
        write!(write, "\n\t[ ")?;
        for item in inner {
            write!(write, "0x{:X}, ", item)?;
        }
        write!(write, "], ")?;
    }
    writeln!(write, "\n];\n")?;

    Ok(())
}
