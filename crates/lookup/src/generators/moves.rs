use crate::{
    utils::utils::{bits, index},
    BOARD_SIZE, COLOR_COUNT,
};

#[rustfmt::skip]
pub fn generate_king_moves() -> [u64; BOARD_SIZE] {
    let mut moves = [0; BOARD_SIZE];

    for rank in 0..8 {
        for file in 0..8 {
            let from = index(rank, file);
            if file > 0 { moves[from] |= bits(rank, file - 1); }
            if file < 7 { moves[from] |= bits(rank, file + 1); }
            if rank > 0 { moves[from] |= bits(rank - 1, file); }
            if rank < 7 { moves[from] |= bits(rank + 1, file); }
            if rank > 0 && file > 0 { moves[from] |= bits(rank - 1, file - 1); }
            if rank > 0 && file < 7 { moves[from] |= bits(rank - 1, file + 1); }
            if rank < 7 && file > 0 { moves[from] |= bits(rank + 1, file - 1); }
            if rank < 7 && file < 7 { moves[from] |= bits(rank + 1, file + 1); }
        }
    }

    moves
}

#[rustfmt::skip]
pub fn generate_knight_moves() -> [u64; BOARD_SIZE] {
    let mut moves = [0; BOARD_SIZE];

    for rank in 0..8 {
        for file in 0..8 {
            let from = index(rank, file);
            if rank > 1 && file > 0 { moves[from] |= bits(rank - 2, file - 1); }
            if rank > 1 && file < 7 { moves[from] |= bits(rank - 2, file + 1); }
            if rank > 0 && file > 1 { moves[from] |= bits(rank - 1, file - 2); }
            if rank > 0 && file < 6 { moves[from] |= bits(rank - 1, file + 2); }
            if rank < 7 && file > 1 { moves[from] |= bits(rank + 1, file - 2); }
            if rank < 7 && file < 6 { moves[from] |= bits(rank + 1, file + 2); }
            if rank < 6 && file > 0 { moves[from] |= bits(rank + 2, file - 1); }
            if rank < 6 && file < 7 { moves[from] |= bits(rank + 2, file + 1); }
        }
    }

    moves
}

#[rustfmt::skip]
pub fn generate_pawn_pushes() -> [[u64; BOARD_SIZE]; COLOR_COUNT] {
    let mut moves = [[0; BOARD_SIZE]; COLOR_COUNT];

    for rank in 0..8 {
        for file in 0..8 {
            let from = index(rank, file);

            if rank > 0 { moves[0][from] |= bits(rank - 1, file); }
            if rank < 7 { moves[1][from] |= bits(rank + 1, file); }

            if rank == 6 { moves[0][from] |= bits(rank - 2, file); }
            if rank == 1 { moves[1][from] |= bits(rank + 2, file); }
        }
    }

    moves
}

#[rustfmt::skip]
pub fn generate_pawn_attacks() -> [[u64; BOARD_SIZE]; COLOR_COUNT] {
    let mut moves = [[0; BOARD_SIZE]; COLOR_COUNT];

    for rank in 0..8 {
        for file in 0..8 {
            let from = index(rank, file);
            if rank > 0 && file > 0 { moves[0][from] |= bits(rank - 1, file - 1); }
            if rank > 0 && file < 7 { moves[0][from] |= bits(rank - 1, file + 1); }

            if rank < 7 && file > 0 { moves[1][from] |= bits(rank + 1, file - 1); }
            if rank < 7 && file < 7 { moves[1][from] |= bits(rank + 1, file + 1); }
        }
    }

    moves
}
