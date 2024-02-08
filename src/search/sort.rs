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

pub fn sort_moves(first: &Move, second: &Move) -> Ordering {
    let first_score = score_move(first);
    let second_score = score_move(second);
    second_score.cmp(&first_score)
}

fn score_move(mov: &Move) -> isize {
    match mov.kind {
        MoveKind::Attack(ref attack) => MVV_LVA[attack.attacked.index()][mov.piece.index()],
        _ => 0,
    }
}
