use base::{board::piece::Piece, r#move::Move};

use super::killers::{
    Killers, KILLER_REDUCTION, KILLER_SCORE, MATE_KILLER_REDUCTION, MATE_KILLER_SCORE,
};

pub(crate) const SCORE_SLICE: usize = std::usize::MAX / 5;

pub(crate) const PV_SCORE: usize = SCORE_SLICE * 5;

pub(crate) const MVV_LVA_SCORE: usize = SCORE_SLICE * 4;

#[rustfmt::skip]
pub(crate) const MVV_LVA: [[usize; Piece::COUNT]; Piece::COUNT] = [
    [0,  0,  0,  0,  0,  0,  0],
    [0, 15, 14, 13, 12, 11, 10],
    [0, 25, 24, 23, 22, 21, 20],
    [0, 35, 34, 33, 32, 31, 30],
    [0, 45, 44, 43, 42, 41, 40],
    [0, 55, 54, 53, 52, 51, 50],
    [0,  0,  0,  0,  0,  0,  0],
];

#[derive(Debug)]
pub(crate) struct ScoredMove {
    mov: Move,
    score: usize,
}

impl ScoredMove {
    pub fn new(mov: Move, score: usize) -> Self {
        Self { mov, score }
    }
}

pub(crate) fn pick_next_move(move_index: usize, moves: &mut Vec<ScoredMove>) -> Move {
    let mut best_index = move_index;
    let mut best_score = 0;

    for index in move_index..moves.len() {
        let move_score = moves[index].score;
        if move_score > best_score {
            best_score = move_score;
            best_index = index;
        }
    }

    moves.swap(move_index, best_index);

    let next_move = moves[move_index].mov;
    next_move
}

pub(crate) fn score_moves(
    moves: Vec<Move>,
    ply: u8,
    pv_move: Option<Move>,
    killers: &Killers,
    mate_killers: &Killers,
) -> Vec<ScoredMove> {
    let mut scored_moves = Vec::with_capacity(moves.len());

    for mov in moves {
        let score = score_move(ply, &mov, &pv_move, killers, mate_killers);
        scored_moves.push(ScoredMove::new(mov, score));
    }

    scored_moves
}

pub(crate) fn score_move(
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

    if mov.is_capture() {
        let captured = mov.captured_piece();
        let piece = mov.piece();

        let mut score = MVV_LVA_SCORE;
        score += MVV_LVA[captured.index()][piece.index()];
        return score;
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
