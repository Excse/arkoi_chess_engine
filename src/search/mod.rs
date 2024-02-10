pub mod sort;

use crate::{board::Board, move_generator::mov::Move};

pub const MAX_DEPTH: usize = 64;
pub const MAX_KILLERS: usize = 2;

pub const CHECKMATE: isize = 1_000_000;
pub const CHECKMATE_PLY: isize = 1_000;
pub const CHECKMATE_MIN: isize = CHECKMATE - (MAX_DEPTH as isize) * CHECKMATE_PLY;
pub const DRAW: isize = 0;

pub const MAX_EVAL: isize = CHECKMATE + 1;
pub const MIN_EVAL: isize = -CHECKMATE - 1;

pub const NULL_DEPTH_REDUCTION: u8 = 2;

#[derive(Debug)]
pub struct KillerMoves {
    pub killer_moves: [[Option<Move>; MAX_KILLERS]; MAX_DEPTH],
}

impl Default for KillerMoves {
    fn default() -> Self {
        KillerMoves {
            killer_moves: [[None; MAX_KILLERS]; MAX_DEPTH],
        }
    }
}

impl KillerMoves {
    pub fn store(&mut self, mov: &Move, ply: u8) {
        let killers = &mut self.killer_moves[ply as usize];

        // We dont want to store the same move twice.
        match &killers[0] {
            Some(killer) if killer == mov => return,
            _ => {}
        }

        killers[1] = killers[0];
        killers[0] = Some(*mov);
    }

    pub fn contains(&self, mov: &Move, ply: u8) -> Option<usize> {
        let killers = &self.killer_moves[ply as usize];

        for index in 0..MAX_KILLERS {
            match &killers[index] {
                Some(killer) if killer == mov => return Some(index),
                _ => {}
            }
        }

        None
    }
}

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

    let mut mate_killer_moves = KillerMoves::default();
    let mut killer_moves = KillerMoves::default();

    let mut parent_pv = Vec::new();
    for depth in 1..=max_depth {
        let start = std::time::Instant::now();
        let eval = negamax(
            board,
            &mut parent_pv,
            depth,
            0,
            MIN_EVAL,
            MAX_EVAL,
            false,
            false,
            &mut killer_moves,
            &mut mate_killer_moves,
        );

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

        // If we alreay found a checkmate we dont need to search deeper,
        // as there can only be a checkmate in more moves. But as we already
        // penalize checkmates at a deeper depth, we just can cut here.
        if eval >= CHECKMATE_MIN {
            break;
        }
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
fn quiescence(
    board: &Board,
    ply: u8,
    mut alpha: isize,
    beta: isize,
    killers: &mut KillerMoves,
    mate_killers: &mut KillerMoves,
) -> isize {
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
    let mut best_eval = alpha;

    // TODO: We need to generate only attacking moves.
    let mut move_state = board.get_legal_moves().unwrap();
    // TODO: Test if this is useful
    if move_state.is_checkmate {
        return -CHECKMATE + (ply as isize * CHECKMATE_PLY);
    }

    // ~~~~~~~~~ MOVE ORDERING ~~~~~~~~~
    // Used to improve the efficiency of the alpha-beta algorithm.
    // Source: https://www.chessprogramming.org/Move_Ordering
    // TODO: Only do capture
    let pv_move = None;
    move_state.moves.sort_unstable_by(|first, second| {
        sort::sort_moves(ply, first, second, &pv_move, killers, mate_killers)
    });
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

        let child_eval = -quiescence(&board, ply + 1, -beta, -alpha, killers, mate_killers);
        best_eval = best_eval.max(child_eval);

        alpha = alpha.max(child_eval);

        // If alpha is greater or equal to beta, we need to make
        // a beta cut-off. All other moves will be worse than the
        // current best move.
        if alpha >= beta {
            // We differentiate between mate and normal killers, as mate killers
            // will have a higher score and thus will be prioritized.
            if alpha >= CHECKMATE_MIN || alpha <= -CHECKMATE_MIN {
                mate_killers.store(&mov, ply);
            } else {
                killers.store(&mov, ply);
            }
            return beta;
        }
    }

    best_eval
}

fn negamax(
    board: &Board,
    parent_pv: &mut Vec<Move>,
    mut depth: u8,
    ply: u8,
    mut alpha: isize,
    beta: isize,
    mut extended: bool,
    do_null_move: bool,
    killers: &mut KillerMoves,
    mate_killers: &mut KillerMoves,
) -> isize {
    // ~~~~~~~~~ CUT-OFF ~~~~~~~~~
    // These are tests which decide if you should stop searching based
    // on the current state of the board.
    // TODO: Add time limitation
    if depth == 0 {
        return quiescence(board, ply + 1, alpha, beta, killers, mate_killers);
    } else if board.halfmoves >= 50 {
        // TODO: Offer a draw when using a different communication protocol
        // like XBoard
        return DRAW;
    } else if board.is_threefold_repetition() {
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

    // ~~~~~~~~~ NULL MOVE PRUNING ~~~~~~~~~
    // Using this pruning technique we check if our position is so
    // good that the opponent could even make a double move without
    // getting a better position.
    //
    // Also we need to limit this techniue so it can't occur two times
    // in a row. Also we disable it if the current depth is too low, as
    // it could lead to a wrong decision.
    //
    // Source: https://www.chessprogramming.org/Null_Move_Pruning
    // TODO: Add zugzwang detection
    if do_null_move && !move_state.is_check && depth >= 5 {
        let mut board = board.clone();
        board.swap_active();

        let null_eval = -negamax(
            &board,
            parent_pv,
            depth - 1 - NULL_DEPTH_REDUCTION,
            ply + 1,
            -beta,
            -beta + 1,
            extended,
            false,
            killers,
            mate_killers,
        );

        if null_eval >= beta {
            return beta;
        }
    }
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    // ~~~~~~~~~ MOVE ORDERING ~~~~~~~~~
    // Used to improve the efficiency of the alpha-beta algorithm.
    // Source: https://www.chessprogramming.org/Move_Ordering
    let pv_move = parent_pv.first().cloned();
    move_state.moves.sort_unstable_by(|first, second| {
        sort::sort_moves(ply, first, second, &pv_move, killers, mate_killers)
    });
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    // The best evaluation found so far.
    let mut best_eval = MIN_EVAL;

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
                true,
                killers,
                mate_killers,
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
                true,
                killers,
                mate_killers,
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
                    true,
                    killers,
                    mate_killers,
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
            // We differentiate between mate and normal killers, as mate killers
            // will have a higher score and thus will be prioritized.
            if alpha >= CHECKMATE_MIN || alpha <= -CHECKMATE_MIN {
                mate_killers.store(&mov, ply);
            } else {
                killers.store(&mov, ply);
            }
            return beta;
        }
    }

    best_eval
}
