use const_for::*;

use crate::{BOARD_SIZE, COLOR_COUNT, utils::{index, bits}};

#[rustfmt::skip]
pub const KING_MOVES: [u64; BOARD_SIZE] = {
    let mut moves = [0; BOARD_SIZE];

    const_for!(rank in 0..8 => {
        const_for!(file in 0..8 => {
            let from = index(rank, file);
            if file > 0 { moves[from] |= bits(rank, file - 1); }
            if file < 7 { moves[from] |= bits(rank, file + 1); }
            if rank > 0 { moves[from] |= bits(rank - 1, file); }
            if rank < 7 { moves[from] |= bits(rank + 1, file); }
            if rank > 0 && file > 0 { moves[from] |= bits(rank - 1, file - 1); }
            if rank > 0 && file < 7 { moves[from] |= bits(rank - 1, file + 1); }
            if rank < 7 && file > 0 { moves[from] |= bits(rank + 1, file - 1); }
            if rank < 7 && file < 7 { moves[from] |= bits(rank + 1, file + 1); }
        });
    });

    moves
};

#[rustfmt::skip]
pub const KNIGHT_MOVES: [u64; BOARD_SIZE] = {
    let mut moves = [0; BOARD_SIZE];

    const_for!(rank in 0..8 => {
        const_for!(file in 0..8 => {
            let from = index(rank, file);
            if rank > 1 && file > 0 { moves[from] |= bits(rank - 2, file - 1); }
            if rank > 1 && file < 7 { moves[from] |= bits(rank - 2, file + 1); }
            if rank > 0 && file > 1 { moves[from] |= bits(rank - 1, file - 2); }
            if rank > 0 && file < 6 { moves[from] |= bits(rank - 1, file + 2); }
            if rank < 7 && file > 1 { moves[from] |= bits(rank + 1, file - 2); }
            if rank < 7 && file < 6 { moves[from] |= bits(rank + 1, file + 2); }
            if rank < 6 && file > 0 { moves[from] |= bits(rank + 2, file - 1); }
            if rank < 6 && file < 7 { moves[from] |= bits(rank + 2, file + 1); }
        });
    });

    moves
};

#[rustfmt::skip]
pub const PAWN_PUSHES: [[u64; BOARD_SIZE]; COLOR_COUNT] = {
    let mut moves = [[0; BOARD_SIZE]; COLOR_COUNT];

    const_for!(rank in 0..8 => {
        const_for!(file in 0..8 => {
            let from = index(rank, file);

            if rank > 0 { moves[0][from] |= bits(rank - 1, file); }
            if rank < 7 { moves[1][from] |= bits(rank + 1, file); }

            if rank == 6 { moves[0][from] |= bits(rank - 2, file); }
            if rank == 1 { moves[1][from] |= bits(rank + 2, file); }
        })
    });

    moves
};

#[rustfmt::skip]
pub const PAWN_ATTACKS: [[u64; BOARD_SIZE]; COLOR_COUNT] = {
    let mut moves = [[0; BOARD_SIZE]; COLOR_COUNT];

    const_for!(rank in 0..8 => {
        const_for!(file in 0..8 => {
            let from = index(rank, file);
            if rank > 0 && file > 0 { moves[0][from] |= bits(rank - 1, file - 1); }
            if rank > 0 && file < 7 { moves[0][from] |= bits(rank - 1, file + 1); }

            if rank < 7 && file > 0 { moves[1][from] |= bits(rank + 1, file - 1); }
            if rank < 7 && file < 7 { moves[1][from] |= bits(rank + 1, file + 1); }
        });
    });

    moves
};
