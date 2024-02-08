pub mod sort;

use crate::{board::Board, move_generator::mov::Move};

pub const CHECKMATE: isize = 40_000;
pub const DRAW: isize = 0;

pub const MAX_EVAL: isize = CHECKMATE + 1;
pub const MIN_EVAL: isize = -CHECKMATE - 1;

fn pesto_evaluation(board: &Board) -> isize {
    let unactive = (!board.active).index();
    let active = board.active.index();

    let midgame_score = board.midgame[active] - board.midgame[unactive];
    let endgame_score = board.endgame[active] - board.endgame[unactive];

    let mut midgame_phase = board.gamephase;
    if midgame_phase > 24 {
        midgame_phase = 24;
    }
    let endgame_phase = 24 - midgame_phase;

    let mut eval = midgame_score * midgame_phase;
    eval += endgame_score * endgame_phase;
    eval /= 24;

    eval
}

pub fn evaluate(board: &Board) -> isize {
    let mut eval = 0;

    eval += pesto_evaluation(board);

    eval
}

pub fn iterative_deepening(board: &Board, max_depth: u8) -> (isize, Option<Move>) {
    let mut best_eval = std::isize::MIN;
    let mut best_move = None;

    for depth in 1..=max_depth {
        let mut pv = Vec::new();

        let start = std::time::Instant::now();
        let eval = negamax(board, depth, 0, MIN_EVAL, MAX_EVAL, &mut pv, false);
        let elapsed = start.elapsed();

        println!(
            "info depth {} score cp {} time {} pv {}",
            depth,
            eval,
            elapsed.as_millis(),
            pv.iter()
                .map(|mov| mov.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        );

        if eval > best_eval {
            best_eval = eval;
            best_move = pv.first().cloned();
        }
    }

    (best_eval, best_move)
}

// By using quiescence search, we can avoid the horizon effect.
// This describes the situation where the search horizon is reached
// and the evaluation states that the position is equal or better,
// even if the position is actually worse.
//
// For that purpose we just evaluate all the captures to reach a
// quiet position. At this point we can evaluate the position and be
// sure that the evaluation is accurate enough.
//
/// By using quiescence search, we can avoid the horizon effect.
/// This describes the situation where the search horizon is reached
/// and the evaluation states that the position is equal or better,
/// even if the position is actually worse.
///
/// For that purpose we just evaluate all the captures to reach a
/// quiet position. At this point we can evaluate the position and be
/// sure that the evaluation is accurate enough.
///
/// Source: https://www.chessprogramming.org/Quiescence_Search
fn quiescence(board: &Board, mut alpha: isize, beta: isize) -> isize {
    let standing_pat = evaluate(board);

    // If the evaluation exceeds the upper bound we just fail hard.
    if standing_pat >= beta {
        return beta;
    }

    // Set the new lower bound if the evaluation is better than the current.
    if standing_pat > alpha {
        alpha = standing_pat;
    }

    // The best evaluation found so far.
    let mut eval = alpha;

    // TODO: We need to generate only attacking moves.
    let move_state = board.get_legal_moves().unwrap();
    for mov in move_state.moves {
        // TODO: This needs to be removed when we can generate only attacking moves.
        if !mov.is_attack() {
            continue;
        }

        // TODO: Make an unmake function as the board is getting too big
        // to be cloned.
        let mut board = board.clone();
        board.make(&mov).unwrap();

        // Create own principal variation line and also call negamax to
        // possibly find a better move.
        let leaf_eval = -quiescence(&board, -beta, -alpha);
        eval = eval.max(leaf_eval);

        // If we found a better move, we need to update the alpha.
        alpha = alpha.max(eval);

        // If alpha is greater or equal to beta, we need to make
        // a beta cut-off. All other moves will be worse than the
        // current best move.
        if alpha >= beta {
            break;
        }
    }

    eval
}

fn negamax(
    board: &Board,
    mut depth: u8,
    ply: u8,
    mut alpha: isize,
    beta: isize,
    parent_pv: &mut Vec<Move>,
    mut extended: bool,
) -> isize {
    // ~~~~~~~~~ CUT-OFF ~~~~~~~~~
    // These are tests which decide if you should stop searching based
    // on the current state of the board.
    // TODO: Add time limitation
    // TODO: Add repetition detection
    if depth == 0 {
        return quiescence(board, alpha, beta);
    } else if board.halfmoves >= 50 {
        // TODO: Offer a draw when using a different communication protocol
        // like XBoard
        return DRAW;
    }
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~

    // ~~~~~~~~ TERMINAL ~~~~~~~~
    // A terminal is a node where the game is over and no legal moves
    // are available anymore.
    // Source: https://www.chessprogramming.org/Terminal_Node
    let mut move_state = board.get_legal_moves().unwrap();
    if move_state.is_stalemate {
        return DRAW;
    } else if move_state.is_checkmate {
        return CHECKMATE + ply as isize;
    }
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~

    // ~~~~~~~~ SELECTIVITY ~~~~~~~~
    // Source: https://www.chessprogramming.org/Selectivity
    if move_state.is_check && extended {
        depth += 1;
        extended = true;
    }
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    // ~~~~~~~~~ MOVE ORDERING ~~~~~~~~~
    // Used to improve the efficiency of the alpha-beta algorithm.
    // Source: https://www.chessprogramming.org/Move_Ordering
    move_state.moves.sort_unstable_by(sort::sort_moves);
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    // The best evaluation found so far.
    let mut eval = std::isize::MIN;

    for mov in move_state.moves {
        // TODO: Make an unmake function as the board is getting too big
        // to be cloned.
        let mut board = board.clone();
        board.make(&mov).unwrap();

        // Create own principal variation line and also call negamax to
        // possibly find a better move.
        let mut child_pv = Vec::new();
        let leaf_eval = -negamax(
            &board,
            depth - 1,
            ply + 1,
            -beta,
            -alpha,
            &mut child_pv,
            extended,
        );

        // Decides if we found a better move.
        eval = eval.max(leaf_eval);

        // If we found a better move, we need to update the alpha value but
        // also the principal variation line.
        if eval > alpha {
            alpha = eval;

            parent_pv.clear();
            parent_pv.push(mov);
            parent_pv.append(&mut child_pv);
        }

        // If alpha is greater or equal to beta, we need to make
        // a beta cut-off. All other moves will be worse than the
        // current best move.
        if alpha >= beta {
            break;
        }
    }

    eval
}
