use std::cmp::Ordering;

use crate::{
    board::piece::Piece,
    move_generator::mov::{Move, MoveKind},
};

use super::killers::{
    Killers, KILLER_REDUCTION, KILLER_SCORE, MATE_KILLER_REDUCTION, MATE_KILLER_SCORE,
};

pub const SCORE_SLICE: usize = std::usize::MAX / 5;

pub const PV_SCORE: usize = SCORE_SLICE * 5;

pub const MVV_LVA_SCORE: usize = SCORE_SLICE * 4;

#[rustfmt::skip]
pub const MVV_LVA: [[usize; Piece::COUNT]; Piece::COUNT] = [
    [15, 14, 13, 12, 11, 10],
    [25, 24, 23, 22, 21, 20],
    [35, 34, 33, 32, 31, 30],
    [45, 44, 43, 42, 41, 40],
    [55, 54, 53, 52, 51, 50],
    [ 0,  0,  0,  0,  0,  0],
];

pub fn sort_moves(
    ply: u8,
    first: &Move,
    second: &Move,
    pv_move: &Option<Move>,
    killers: &Killers,
    mate_killers: &Killers,
) -> Ordering {
    let first_score = score_move(ply, first, pv_move, killers, mate_killers);
    let second_score = score_move(ply, second, pv_move, killers, mate_killers);
    second_score.cmp(&first_score)
}

fn score_move(
    ply: u8,
    mov: &Move,
    pv_move: &Option<Move>,
    killers: &Killers,
    mate_killers: &Killers,
) -> usize {
    if let Some(pv) = pv_move {
        if mov == pv {
            return PV_SCORE;
        }
    }

    match &mov.kind {
        MoveKind::Attack(attack) => {
            let mut score = MVV_LVA_SCORE;
            score += MVV_LVA[attack.attacked.index()][mov.piece.index()];
            return score;
        }
        MoveKind::EnPassant(_) => {
            let mut score = MVV_LVA_SCORE;
            score += MVV_LVA[Piece::Pawn.index()][Piece::Pawn.index()];
            return score;
        }
        MoveKind::Promotion(promotion) => {
            if let Some(attacked) = promotion.attacked {
                let mut score = MVV_LVA_SCORE;
                score += MVV_LVA[attacked.index()][mov.piece.index()];
                return score;
            }
        }
        _ => {}
    }

    if let Some(index) = mate_killers.contains(mov, ply) {
        let mut score = MATE_KILLER_SCORE;
        score -= index * MATE_KILLER_REDUCTION;
        return score;
    }

    if let Some(index) = killers.contains(mov, ply) {
        let mut score = KILLER_SCORE;
        score -= index * KILLER_REDUCTION;
        return score;
    }

    0
}
