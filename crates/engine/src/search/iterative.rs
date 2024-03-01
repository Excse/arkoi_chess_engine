use std::time::Instant;

use base::{board::Board, r#move::Move};

use crate::{
    generator::{error::MoveGeneratorError, AllMoves, MoveGenerator},
    hashtable::TranspositionTable,
    search::{negamax::negamax, CHECKMATE_MIN, MAX_EVAL, MIN_EVAL},
};

use super::{
    communication::{BestMove, Info, Score},
    error::SearchError,
    sort::{pick_next_move, score_moves},
    SearchInfo, SearchStats, StopReason, CHECKMATE,
};

pub(crate) fn iterative_deepening(
    cache: &TranspositionTable,
    mut info: SearchInfo,
) -> Result<(), SearchError> {
    let mut best_move = None;
    for depth in 1..=info.max_depth {
        let start = Instant::now();

        let mut stats = SearchStats::new(depth);

        // TODO: Use the given moves: info.moves()
        let result = negamax(
            cache, &mut info, &mut stats, MIN_EVAL, MAX_EVAL, false, false,
        );
        let best_eval = match result {
            Ok(result) => result,
            Err(StopReason::TimeUp)
            | Err(StopReason::NodesExceeded)
            | Err(StopReason::ForcedStop) => break,
        };

        let elapsed = start.elapsed();
        let nodes_per_second = (stats.nodes as f64 / elapsed.as_secs_f64()) as u64;
        info.accumulated_nodes += stats.nodes;

        let score = if best_eval.abs() >= CHECKMATE_MIN {
            let ply = CHECKMATE - best_eval.abs();
            let is_odd = ply % 2 == 1;

            let moves = if is_odd { (ply + 1) / 2 } else { ply / 2 };
            Score::Mate(moves * best_eval.signum())
        } else {
            Score::Centipawns(best_eval)
        };

        let pv_line = get_pv_line(&mut info, cache, depth)?;
        best_move = pv_line.get(0).cloned();

        let hashfull = cache.full_percentage();

        info.sender.send(
            Info::new()
                .depth(depth)
                .seldepth(stats.max_ply)
                .time(elapsed.as_millis())
                .hashfull(hashfull)
                .score(score)
                .nodes(stats.nodes)
                .pv(pv_line)
                .nps(nodes_per_second)
                .build(),
        )?;

        // If we are not in infinite mode, we want to check if we have enough
        // time for the next iteration. We approximate the time the next iteration
        // will take by doubling the time of the current iteration.
        // This will safe us some time, as we can stop the search earlier.
        if !info.infinite {
            let elapsed = stats.start_time.elapsed().as_millis();
            if elapsed * 2 > info.time_frame.move_time {
                break;
            }
        }

        // If we alreay found a checkmate we dont need to search deeper,
        // as there can only be a checkmate in more moves. But as we already
        // penalize checkmates at a deeper depth, we just can cut here.
        if !info.infinite && best_eval >= CHECKMATE_MIN {
            break;
        }
    }

    let best_move = match best_move {
        Some(mov) => mov,
        None => {
            // If there is no best move, choose a random move as we did not
            // have enough time to search the best move.
            let move_generator = MoveGenerator::<AllMoves>::new(&info.board);
            let mut stats = SearchStats::new(0);

            let moves = move_generator.collect::<Vec<Move>>();
            let mut scored_moves = score_moves(&info, &mut stats, moves, None);
            let next_move = pick_next_move(0, &mut scored_moves);
            next_move
        }
    };

    info.sender.send(BestMove::new(best_move))?;

    Ok(())
}

pub(crate) fn get_pv_line(
    info: &mut SearchInfo,
    cache: &TranspositionTable,
    max_depth: u8,
) -> Result<Vec<Move>, MoveGeneratorError> {
    let mut pv = Vec::new();

    let mut board = info.board.clone();
    for _ in 0..max_depth {
        let pv_move = match probe_pv_move(&board, cache)? {
            Some(mov) => mov,
            None => break,
        };

        board.make(pv_move);
        pv.push(pv_move);
    }

    Ok(pv)
}

pub(crate) fn probe_pv_move(
    board: &Board,
    cache: &TranspositionTable,
) -> Result<Option<Move>, MoveGeneratorError> {
    let entry = match cache.probe(board.hash()) {
        Some(entry) => entry,
        None => return Ok(None),
    };

    let best_move = match entry.best_move() {
        Some(mov) => mov,
        None => return Ok(None),
    };

    if !move_exists(board, best_move)? {
        return Ok(None);
    }

    Ok(Some(best_move))
}

pub(crate) fn move_exists(board: &Board, given: Move) -> Result<bool, MoveGeneratorError> {
    let move_generator = MoveGenerator::<AllMoves>::new(board);
    for mov in move_generator {
        if mov == given {
            return Ok(true);
        }
    }

    return Ok(false);
}
