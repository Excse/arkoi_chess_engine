use crate::{
    board::Board,
    hashtable::{
        transposition::{TranspositionEntry, TranspositionFlag},
        HashTable,
    },
};

use super::{
    error::SearchError,
    killers::Killers,
    quiescence::quiescence,
    sort::{pick_next_move, score_moves},
    TimeFrame, CHECKMATE, CHECKMATE_MIN, DRAW, MIN_EVAL, NULL_DEPTH_REDUCTION,
};

pub fn negamax(
    board: &mut Board,
    cache: &mut HashTable<TranspositionEntry>,
    killers: &mut Killers,
    mate_killers: &mut Killers,
    nodes: &mut usize,
    time_frame: &TimeFrame,
    mut depth: u8,
    ply: u8,
    mut alpha: isize,
    mut beta: isize,
    mut extended: bool,
    do_null_move: bool,
) -> Result<isize, SearchError> {
    *nodes += 1;

    time_frame.is_time_up()?;

    let mut hash_move = None;
    if let Some(entry) = cache.probe(board.hash()) {
        if entry.depth >= depth {
            if entry.best_move.is_some() {
                hash_move = entry.best_move;
            }

            match entry.flag {
                TranspositionFlag::Exact => return Ok(entry.eval),
                TranspositionFlag::LowerBound => alpha = alpha.max(entry.eval),
                TranspositionFlag::UpperBound => beta = beta.min(entry.eval),
            }

            *nodes += entry.nodes;

            if alpha >= beta {
                return Ok(entry.eval);
            }
        }
    }

    // ~~~~~~~~~ MATE DISTANCE PRUNING ~~~~~~~~~
    // TODO: Add a description
    let mate_value = CHECKMATE - ply as isize;
    if mate_value < beta {
        beta = mate_value;
        if alpha >= mate_value {
            return Ok(mate_value);
        }
    }

    if -mate_value > alpha {
        alpha = -mate_value;

        if beta <= -mate_value {
            return Ok(-mate_value);
        }
    }

    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

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
            time_frame,
            ply + 1,
            alpha,
            beta,
        )?;
        *nodes += visited_nodes;
        return Ok(eval);
    } else if board.halfmoves() >= 100 {
        // TODO: Offer a draw when using a different communication protocol
        // like XBoard
        return Ok(DRAW);
    } else if board.is_threefold_repetition() {
        return Ok(DRAW);
    }
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~

    // ~~~~~~~~ TERMINAL ~~~~~~~~
    // A terminal is a node where the game is over and no legal moves
    // are available anymore.
    // Source: https://www.chessprogramming.org/Terminal_Node
    let move_state = board.get_legal_moves().unwrap();
    if move_state.is_stalemate {
        return Ok(DRAW);
    } else if move_state.is_checkmate {
        return Ok(-CHECKMATE + ply as isize);
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
        board.make_null();

        let result = negamax(
            board,
            cache,
            killers,
            mate_killers,
            nodes,
            time_frame,
            depth - 1 - NULL_DEPTH_REDUCTION,
            ply + 1,
            -beta,
            -beta + 1,
            extended,
            false,
        );
        let null_eval = match result {
            Ok(eval) => -eval,
            Err(error) => {
                board.unmake_null();
                return Err(error);
            }
        };

        board.unmake_null();

        if null_eval >= beta {
            return Ok(beta);
        }
    }
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    // ~~~~~~~~~ MOVE ORDERING ~~~~~~~~~
    // Used to improve the efficiency of the alpha-beta algorithm.
    // Source: https://www.chessprogramming.org/Move_Ordering
    let mut scored_moves = score_moves(move_state.moves, ply, hash_move, killers, mate_killers);
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let mut best_move = hash_move;
    let mut best_eval = MIN_EVAL;

    let mut visited_nodes = 0;
    for move_index in 0..scored_moves.len() {
        let next_move = pick_next_move(move_index, &mut scored_moves);

        board.make(&next_move);

        // The evaluation of the current move.
        let mut child_eval;

        // As we assume that the first move is the best one, we only want to
        // search this specific move with the full window.
        if move_index == 0 {
            let result = negamax(
                board,
                cache,
                killers,
                mate_killers,
                &mut visited_nodes,
                time_frame,
                depth - 1,
                ply + 1,
                -beta,
                -alpha,
                extended,
                true,
            );
            child_eval = match result {
                Ok(eval) => -eval,
                Err(error) => {
                    board.unmake(&next_move);
                    return Err(error);
                }
            };
        } else {
            // TODO: Remove the magic numbers
            if move_index >= 4 && depth >= 3 && !move_state.is_check && !next_move.is_tactical() {
                let result = negamax(
                    board,
                    cache,
                    killers,
                    mate_killers,
                    &mut visited_nodes,
                    time_frame,
                    // TODO: Calculate the depth reduction
                    depth - 2,
                    ply + 1,
                    -(alpha + 1),
                    -alpha,
                    extended,
                    true,
                );
                child_eval = match result {
                    Ok(eval) => -eval,
                    Err(error) => {
                        board.unmake(&next_move);
                        return Err(error);
                    }
                };
            } else {
                child_eval = alpha + 1;
            }

            if child_eval > alpha {
                // If its not the principal variation move test that
                // it is not a better move by using the null window search.
                let result = negamax(
                    board,
                    cache,
                    killers,
                    mate_killers,
                    &mut visited_nodes,
                    time_frame,
                    depth - 1,
                    ply + 1,
                    -alpha - 1,
                    -alpha,
                    extended,
                    true,
                );
                child_eval = match result {
                    Ok(eval) => -eval,
                    Err(error) => {
                        board.unmake(&next_move);
                        return Err(error);
                    }
                };

                // If the test failed, we need to research the move with the
                // full window.
                if child_eval > alpha && child_eval < beta {
                    let result = negamax(
                        board,
                        cache,
                        killers,
                        mate_killers,
                        &mut visited_nodes,
                        time_frame,
                        depth - 1,
                        ply + 1,
                        -beta,
                        -alpha,
                        extended,
                        true,
                    );
                    child_eval = match result {
                        Ok(eval) => -eval,
                        Err(error) => {
                            board.unmake(&next_move);
                            return Err(error);
                        }
                    }
                }
            }
        }

        board.unmake(&next_move);

        best_eval = best_eval.max(child_eval);

        // If we found a better move, we need to update the alpha and the
        // best move.
        if best_eval > alpha {
            alpha = best_eval;
            best_move = Some(next_move);
        }

        // If alpha is greater or equal to beta, we need to make
        // a beta cut-off. All other moves will be worse than the
        // current best move.
        if alpha >= beta {
            // Only quiet moves can be killers.
            if !next_move.is_capture() {
                // We differentiate between mate and normal killers, as mate killers
                // will have a higher score and thus will be prioritized.
                if alpha.abs() >= CHECKMATE_MIN {
                    mate_killers.store(&next_move, ply);
                } else {
                    killers.store(&next_move, ply);
                }
            }

            break;
        }
    }

    let flag = if best_eval >= beta {
        TranspositionFlag::LowerBound
    } else if best_eval <= alpha {
        TranspositionFlag::UpperBound
    } else {
        TranspositionFlag::Exact
    };

    *nodes += visited_nodes;
    cache.store(TranspositionEntry::new(
        board.hash(),
        depth,
        flag,
        best_eval,
        visited_nodes,
        best_move,
    ));

    Ok(best_eval)
}
