pub mod sort;

use crate::{board::Board, move_generator::mov::Move};

pub const CHECKMATE: isize = 100_000;
pub const CHECKMATE_PLY: isize = 1_000;
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

pub fn iterative_deepening(board: &Board, max_depth: u8) -> Option<Move> {
    let mut best_move = None;

    let mut parent_pv = Vec::new();
    for depth in 1..=max_depth {
        let start = std::time::Instant::now();
        let eval = negamax(board, &mut parent_pv, depth, 0, MIN_EVAL, MAX_EVAL, false);
        let elapsed = start.elapsed();

        println!(
            "info depth {} score cp {} time {} pv {}",
            depth,
            eval,
            elapsed.as_millis(),
            parent_pv
                .iter()
                .map(|mov| mov.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        );

        best_move = parent_pv.first().cloned();
    }

    best_move
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
fn quiescence(board: &Board, parent_pv: &mut Vec<Move>, mut alpha: isize, beta: isize) -> isize {
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
    let mut move_state = board.get_legal_moves().unwrap();

    // ~~~~~~~~~ MOVE ORDERING ~~~~~~~~~
    // Used to improve the efficiency of the alpha-beta algorithm.
    // Source: https://www.chessprogramming.org/Move_Ordering
    // TODO: Only do capture & pv move ordering
    let pv_move = parent_pv.first().cloned();
    move_state
        .moves
        .sort_unstable_by(|first, second| sort::sort_moves(first, second, &pv_move));
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

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
        let leaf_eval = -quiescence(&board, parent_pv, -beta, -alpha);
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
    parent_pv: &mut Vec<Move>,
    mut depth: u8,
    ply: u8,
    mut alpha: isize,
    beta: isize,
    mut extended: bool,
) -> isize {
    // ~~~~~~~~~ CUT-OFF ~~~~~~~~~
    // These are tests which decide if you should stop searching based
    // on the current state of the board.
    // TODO: Add time limitation
    // TODO: Add repetition detection
    if depth == 0 {
        return quiescence(board, parent_pv, alpha, beta);
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
        return -CHECKMATE + (ply as isize * CHECKMATE_PLY);
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
    let pv_move = parent_pv.first().cloned();
    move_state
        .moves
        .sort_unstable_by(|first, second| sort::sort_moves(first, second, &pv_move));
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    // The best evaluation found so far.
    let mut best_eval = std::isize::MIN;

    // ~~~~~~~~~ PRINCIPAL VARIATION SEARCH ~~~~~~~~~
    // As we already sorted the moves and the first move is the one from the
    // principal variation line, we can assume that it is the best move to take.
    //
    //
    // Source: https://www.chessprogramming.org/Principal_Variation_Search
    let mut search_pv = true;
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    for mov in move_state.moves {
        // TODO: Make an unmake function as the board is getting too big
        // to be cloned.
        let mut board = board.clone();
        board.make(&mov).unwrap();

        // Create own principal variation line and also call negamax to
        // possibly find a better move.
        let mut child_pv = Vec::new();

        // The evaluation of the current move.
        let mut child_eval;

        // As we assume that the first move is the best one, we only want to
        // search this specific move with the full window.
        if search_pv {
            child_eval = -negamax(
                &board,
                &mut child_pv,
                depth - 1,
                ply + 1,
                -beta,
                -alpha,
                extended,
            );

            // We need to reset this so we can move on with the
            // null window search for the other moves.
            search_pv = false;
        } else {
            // If its not the principal variation move test that
            // it is not a better move by using the null window search.
            child_eval = -negamax(
                &board,
                &mut child_pv,
                depth - 1,
                ply + 1,
                -alpha - 1,
                -alpha,
                extended,
            );

            // If the test failed, we need to research the move with the
            // full window.
            if child_eval > alpha && child_eval < beta {
                child_eval = -negamax(
                    &board,
                    &mut child_pv,
                    depth - 1,
                    ply + 1,
                    -beta,
                    -alpha,
                    extended,
                );
            }
        }

        // Decides if we found a better move.
        best_eval = best_eval.max(child_eval);

        // If we found a better move, we need to update the alpha value but
        // also the principal variation line.
        if best_eval > alpha {
            alpha = best_eval;

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

    best_eval
}
