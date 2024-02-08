use std::cmp::Ordering;

use crate::{
    board::piece::Piece,
    move_generator::mov::{Move, MoveKind},
};

#[rustfmt::skip]
pub const MVV_LVA: [[isize; Piece::COUNT]; Piece::COUNT] = [
    [10, 11, 12, 13, 14, 15],
    [20, 21, 22, 23, 24, 25],
    [30, 31, 32, 33, 34, 35],
    [40, 41, 42, 43, 44, 45],
    [50, 51, 52, 53, 54, 55],
    [0,   0,  0,  0,  0,  0],
];

pub fn sort_moves(first: &Move, second: &Move, pv_move: &Option<Move>) -> Ordering {
    let first_score = score_move(first, pv_move);
    let second_score = score_move(second, pv_move);
    second_score.cmp(&first_score)
}

fn score_move(mov: &Move, pv_move: &Option<Move>) -> isize {
    if let Some(pv) = pv_move {
        if mov == pv {
            return 100_000;
        }
    }

    match mov.kind {
        MoveKind::Attack(ref attack) => MVV_LVA[attack.attacked.index()][mov.piece.index()],
        _ => 0,
    }
}
