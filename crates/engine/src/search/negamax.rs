use base::r#move::Move;

use crate::{
    evaluation::evaluate,
    generator::{AllMoves, MoveGenerator},
    hashtable::{
        entry::{TranspositionEntry, TranspositionFlag},
        TranspositionTable,
    },
};

use super::{
    communication::{Info, SearchSender},
    quiescence::quiescence,
    should_stop_search,
    sort::{pick_next_move, score_moves},
    SearchInfo, SearchStats, StopReason, CHECKMATE, CHECKMATE_MIN, CHECK_TERMINATION, DRAW,
    MAX_EVAL, MIN_EVAL, NULL_DEPTH_REDUCTION, SEND_STATS,
};

pub(crate) fn negamax<S: SearchSender>(
    cache: &TranspositionTable,
    info: &mut SearchInfo<S>,
    stats: &mut SearchStats,
    mut alpha: i32,
    mut beta: i32,
    mut extended: bool,
    do_null_move: bool,
    can_futility_prune: bool,
) -> Result<i32, StopReason> {
    stats.nodes += 1;
    if stats.nodes & CHECK_TERMINATION == 0 {
        should_stop_search(info, stats)?;
    }

    // ~~~~~~~~~ CUT-OFF ~~~~~~~~~
    // These are tests which decide if you should stop searching based
    // on the current state of the board.
    if info.board.is_draw() {
        return Ok(DRAW);
    } else if stats.is_leaf() {
        // Corrects the node count, as it will be increased by one inside
        // the quiescence search.
        stats.nodes -= 1;

        stats.increase_ply();
        let result = quiescence(cache, info, stats, alpha, beta);
        stats.decrease_ply();

        return Ok(result?);
    }
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~

    let mut hash_move = None;
    if let Some(entry) = cache.probe(info.board.hash()) {
        if entry.depth() >= stats.depth() {
            if let Some(best_move) = entry.best_move() {
                hash_move = Some(best_move);
            }

            let eval = entry.eval();
            match entry.flag() {
                TranspositionFlag::Exact => return Ok(eval),
                TranspositionFlag::LowerBound => alpha = alpha.max(eval),
                TranspositionFlag::UpperBound => beta = beta.min(eval),
            }

            if alpha >= beta {
                return Ok(eval);
            }
        }
    }

    if can_futility_prune && !info.board.is_check() {
        let futility_margin = match stats.depth() {
            1 => 300,
            2 => 500,
            _ => MAX_EVAL,
        };

        let eval = evaluate(&info.board, info.board.active());
        if eval + futility_margin <= alpha {
            return Ok(alpha);
        }
    }

    // ~~~~~~~~ TERMINAL ~~~~~~~~
    // A terminal is a node where the game is over and no legal moves
    // are available anymore.
    // Source: https://www.chessprogramming.org/Terminal_Node
    let move_generator = MoveGenerator::<AllMoves>::new(&info.board);
    if move_generator.is_stalemate(&info.board) {
        return Ok(DRAW);
    } else if move_generator.is_checkmate(&info.board) {
        return Ok(-CHECKMATE + stats.ply() as i32);
    }
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~

    // ~~~~~~~~ SELECTIVITY ~~~~~~~~
    // Source: https://www.chessprogramming.org/Selectivity
    if info.board.is_check() && extended {
        stats.extend_search();
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
    if do_null_move && !info.board.is_check() && stats.depth() >= 5 {
        info.board.make_null();

        stats.make_search(NULL_DEPTH_REDUCTION);
        let result = negamax(cache, info, stats, -beta, -beta + 1, extended, false, false);
        stats.unmake_search(NULL_DEPTH_REDUCTION);

        info.board.unmake_null();

        let null_eval = -result?;
        if null_eval >= beta {
            return Ok(beta);
        }
    }
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    // ~~~~~~~~~ MOVE ORDERING ~~~~~~~~~
    // Used to improve the efficiency of the alpha-beta algorithm.
    // Source: https://www.chessprogramming.org/Move_Ordering
    let moves = move_generator.collect::<Vec<Move>>();
    let mut scored_moves = score_moves(info, stats, moves, hash_move);
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    // TODO: Make this better
    if stats.nodes & SEND_STATS == 0 {
        let elapsed = stats.start_time.elapsed();
        let nps = (stats.nodes as f64 / elapsed.as_secs_f64()) as u64;
        // TODO: Remove the unwrap
        info.sender
            .send(
                Info::new()
                    .time(elapsed.as_millis())
                    .nodes(stats.nodes)
                    .hashfull(cache.full_percentage())
                    .nps(nps)
                    .build(),
            )
            .unwrap();
    }

    let mut flag = TranspositionFlag::UpperBound;
    let mut best_move = hash_move;
    let mut best_eval = MIN_EVAL;

    for move_index in 0..scored_moves.len() {
        let next_move = pick_next_move(move_index, &mut scored_moves);

        info.board.make(next_move);

        let can_futility_prune = !info.board.is_check() && !next_move.is_capture();

        // The evaluation of the current move.
        let mut child_eval;

        // As we assume that the first move is the best one, we only want to
        // search this specific move with the full window.
        if move_index == 0 {
            stats.make_search(1);
            let result = negamax(
                cache,
                info,
                stats,
                -beta,
                -alpha,
                extended,
                true,
                can_futility_prune,
            );
            stats.unmake_search(1);

            if let Err(error) = result {
                info.board.unmake(next_move);
                return Err(error);
            }

            child_eval = -result.unwrap();
        } else {
            if move_index >= 4
                && stats.depth() >= 4
                && !info.board.is_check()
                && !next_move.is_tactical()
            {
                // TODO: Calculate the depth reduction
                stats.make_search(3);
                let result = negamax(
                    cache,
                    info,
                    stats,
                    -(alpha + 1),
                    -alpha,
                    extended,
                    true,
                    can_futility_prune,
                );
                stats.unmake_search(3);

                if let Err(error) = result {
                    info.board.unmake(next_move);
                    return Err(error);
                }

                child_eval = -result.unwrap();
            } else {
                child_eval = alpha + 1;
            }

            if child_eval > alpha {
                // If its not the principal variation move test that
                // it is not a better move by using the null window search.
                stats.make_search(1);
                let result = negamax(
                    cache,
                    info,
                    stats,
                    -alpha - 1,
                    -alpha,
                    extended,
                    true,
                    can_futility_prune,
                );
                stats.unmake_search(1);

                if let Err(error) = result {
                    info.board.unmake(next_move);
                    return Err(error);
                }

                child_eval = -result.unwrap();

                // If the test failed, we need to research the move with the
                // full window.
                if child_eval > alpha && child_eval < beta {
                    stats.make_search(1);
                    let result = negamax(
                        cache,
                        info,
                        stats,
                        -beta,
                        -alpha,
                        extended,
                        true,
                        can_futility_prune,
                    );
                    stats.unmake_search(1);

                    if let Err(error) = result {
                        info.board.unmake(next_move);
                        return Err(error);
                    }

                    child_eval = -result.unwrap();
                }
            }
        }

        info.board.unmake(next_move);

        best_eval = best_eval.max(child_eval);

        if best_eval <= alpha {
            continue;
        }

        flag = TranspositionFlag::Exact;
        best_move = Some(next_move);
        alpha = best_eval;

        // We only want to store non pv nodes as well as non captures.
        // The reason behind this is, that the sorting of captures and pv
        // nodes is already done by other mechanisms.
        if move_index != 0 && !next_move.is_capture() {
            let color = info.board.active().index();
            let from = next_move.from().index() as usize;
            let to = next_move.to().index() as usize;
            info.history[color][from][to] += 1usize << stats.depth();
        }

        if alpha < beta {
            continue;
        }

        // We only want to store non pv nodes as well as non captures.
        // The reason behind this is, that the sorting of captures and pv
        // nodes is already done by other mechanisms.
        if move_index != 0 && !next_move.is_capture() {
            // We differentiate between mate and normal killers, as mate killers
            // will have a higher score and thus will be prioritized.
            if alpha.abs() >= CHECKMATE_MIN {
                info.mate_killers.store(&next_move, stats.ply());
            } else {
                info.killers.store(&next_move, stats.ply());
            }
        }

        flag = TranspositionFlag::LowerBound;
        break;
    }

    cache.store(
        info.board.hash(),
        TranspositionEntry::new(stats.depth(), flag, best_eval, best_move),
    );

    Ok(best_eval)
}
