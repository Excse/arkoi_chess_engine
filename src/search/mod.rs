pub mod killers;
pub mod sort;

use crate::{
    board::Board,
    hashtable::{
        transposition::{TranspositionEntry, TranspositionFlag},
        HashTable,
    },
    move_generator::mov::Move,
};

use self::killers::Killers;

pub const MAX_DEPTH: usize = 64;

pub const CHECKMATE: isize = 1_000_000;
pub const CHECKMATE_MIN: isize = CHECKMATE - MAX_DEPTH as isize;
pub const DRAW: isize = 0;

pub const MAX_EVAL: isize = CHECKMATE + 1;
pub const MIN_EVAL: isize = -CHECKMATE - 1;

pub const NULL_DEPTH_REDUCTION: u8 = 2;

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

pub fn iterative_deepening(
    board: &Board,
    cache: &mut HashTable<TranspositionEntry>,
    max_depth: u8,
) -> Option<Move> {
    let mut best_move = None;

    let mut mate_killer_moves = Killers::default();
    let mut killer_moves = Killers::default();

    let mut last_nodes = 0;
    let mut eval = 0;

    let mut parent_pv = Vec::new();
    for depth in 1..=max_depth {
        let start = std::time::Instant::now();

        let mut nodes = 0;
        eval = mtdf(
            board,
            cache,
            &mut parent_pv,
            &mut killer_moves,
            &mut mate_killer_moves,
            &mut nodes,
            eval,
            depth,
        );

        let elapsed = start.elapsed();

        let nodes_per_second = (nodes as f64 / elapsed.as_secs_f64()) as usize;
        let branch_factor = nodes as f64 / last_nodes as f64;
        last_nodes = nodes;

        println!(
            "info depth {} score cp {} time {} nodes {} nps {:.2} bf {:.2} pv {}",
            depth,
            eval,
            elapsed.as_millis(),
            nodes,
            nodes_per_second,
            branch_factor,
            parent_pv
                .iter()
                .map(|mov| mov.to_string())
                .collect::<Vec<String>>()
                .join(" "),
        );

        best_move = parent_pv.first().cloned();

        // If we alreay found a checkmate we dont need to search deeper,
        // as there can only be a checkmate in more moves. But as we already
        // penalize checkmates at a deeper depth, we just can cut here.
        if eval >= CHECKMATE_MIN {
            break;
        }

        // TODO: Give up if we are in a definite checkmate
    }

    best_move
}

pub fn mtdf(
    board: &Board,
    cache: &mut HashTable<TranspositionEntry>,
    parent_pv: &mut Vec<Move>,
    killers: &mut Killers,
    mate_killers: &mut Killers,
    nodes: &mut usize,
    guess: isize,
    depth: u8,
) -> isize {
    let mut upperbound = MAX_EVAL;
    let mut lowerbound = MIN_EVAL;

    let mut eval = guess;
    while lowerbound < upperbound {
        let beta = if eval == lowerbound { eval + 1 } else { eval };

        eval = negamax(
            board,
            cache,
            parent_pv,
            killers,
            mate_killers,
            nodes,
            depth,
            0,
            beta - 1,
            beta,
            false,
            false,
        );

        if beta < upperbound {
            upperbound = beta;
        } else {
            lowerbound = beta;
        }
    }

    eval
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
    killers: &mut Killers,
    mate_killers: &mut Killers,
    nodes: &mut usize,
    ply: u8,
    mut alpha: isize,
    beta: isize,
) -> isize {
    *nodes += 1;

    let standing_pat = evaluate(board);

    // If the evaluation exceeds the upper bound we just fail hard.
    if standing_pat >= beta {
        return beta;
    }

    // Set the new lower bound if the evaluation is better than the current.
    if standing_pat > alpha {
        alpha = standing_pat;
    }

    // TODO: We need to generate only attacking moves.
    let mut move_state = board.get_legal_moves().unwrap();
    // TODO: Test if this is useful
    if move_state.is_checkmate {
        return -CHECKMATE + ply as isize;
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

        let child_eval = -quiescence(&board, killers, mate_killers, nodes, ply + 1, -beta, -alpha);
        alpha = alpha.max(child_eval);

        // If alpha is greater or equal to beta, we need to make
        // a beta cut-off. All other moves will be worse than the
        // current best move.
        if alpha >= beta {
            // We differentiate between mate and normal killers, as mate killers
            // will have a higher score and thus will be prioritized.
            if alpha.abs() >= CHECKMATE_MIN {
                mate_killers.store(&mov, ply);
            } else {
                killers.store(&mov, ply);
            }

            return beta;
        }
    }

    alpha
}

fn negamax(
    board: &Board,
    cache: &mut HashTable<TranspositionEntry>,
    parent_pv: &mut Vec<Move>,
    killers: &mut Killers,
    mate_killers: &mut Killers,
    nodes: &mut usize,
    mut depth: u8,
    ply: u8,
    mut alpha: isize,
    mut beta: isize,
    mut extended: bool,
    do_null_move: bool,
) -> isize {
    *nodes += 1;

    // ~~~~~~~~~ MATE DISTANCE PRUNING ~~~~~~~~~
    // TODO: Add a description
    let mate_value = CHECKMATE - ply as isize;
    if mate_value < beta {
        beta = mate_value;
        if alpha >= mate_value {
            return mate_value;
        }
    }

    if -mate_value > alpha {
        alpha = -mate_value;

        if beta <= -mate_value {
            return -mate_value;
        }
    }
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    if board.halfmoves >= 50 {
        // TODO: Offer a draw when using a different communication protocol
        // like XBoard
        return DRAW;
    } else if board.is_threefold_repetition() {
        return DRAW;
    }

    if let Some(entry) = cache.probe(board.hash) {
        if entry.depth >= depth {
            match entry.flag {
                TranspositionFlag::Exact => return entry.eval,
                TranspositionFlag::LowerBound => alpha = alpha.max(entry.eval),
                TranspositionFlag::UpperBound => beta = beta.min(entry.eval),
            }

            *nodes += entry.nodes;

            if alpha >= beta {
                return entry.eval;
            }
        }
    }

    // ~~~~~~~~~ CUT-OFF ~~~~~~~~~
    // These are tests which decide if you should stop searching based
    // on the current state of the board.
    // TODO: Add time limitation
    if depth == 0 {
        let mut visited_nodes = 0;
        let eval = quiescence(
            board,
            killers,
            mate_killers,
            &mut visited_nodes,
            ply + 1,
            alpha,
            beta,
        );
        *nodes += visited_nodes;
        store(board, cache, depth, alpha, beta, eval, visited_nodes);
        return eval;
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
        return -CHECKMATE + ply as isize;
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
        board.make_null();

        let null_eval = -negamax(
            &board,
            cache,
            parent_pv,
            killers,
            mate_killers,
            nodes,
            depth - 1 - NULL_DEPTH_REDUCTION,
            ply + 1,
            -beta,
            -beta + 1,
            extended,
            false,
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

    let mut best_eval = MIN_EVAL;
    let mut visited_nodes = 0;

    for mov in move_state.moves {
        // TODO: Make an unmake function as the board is getting too big
        // to be cloned.
        let mut board = board.clone();
        board.make(&mov).unwrap();

        // Create own principal variation line and also call negamax to
        // possibly find a better move.
        let mut child_pv = Vec::new();

        // The evaluation of the current move.
        let child_eval = -negamax(
            &board,
            cache,
            &mut child_pv,
            killers,
            mate_killers,
            &mut visited_nodes,
            depth - 1,
            ply + 1,
            -beta,
            -alpha,
            extended,
            true,
        );

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
            if alpha.abs() >= CHECKMATE_MIN {
                mate_killers.store(&mov, ply);
            } else {
                killers.store(&mov, ply);
            }

            break;
        }
    }

    store(board, cache, depth, alpha, beta, best_eval, visited_nodes);
    *nodes += visited_nodes;

    best_eval
}

pub fn store(
    board: &Board,
    cache: &mut HashTable<TranspositionEntry>,
    depth: u8,
    alpha: isize,
    beta: isize,
    eval: isize,
    nodes: usize,
) {
    let flag = if eval >= beta {
        TranspositionFlag::LowerBound
    } else if eval <= alpha {
        TranspositionFlag::UpperBound
    } else {
        TranspositionFlag::Exact
    };

    cache.store(TranspositionEntry::new(
        board.hash, depth, flag, eval, nodes,
    ));
}
