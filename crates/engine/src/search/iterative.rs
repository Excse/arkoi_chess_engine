use std::{fmt::Write, time::Instant};

use base::r#move::Move;

use crate::{
    generator::{error::MoveGeneratorError, MoveGenerator},
    hashtable::TranspositionTable,
    search::{negamax::negamax, CHECKMATE_MIN, MAX_EVAL, MIN_EVAL},
};

use super::{
    error::SearchError,
    sort::{pick_next_move, score_moves},
    SearchInfo, SearchStats, StopReason,
};

pub(crate) fn iterative_deepening<W: Write>(
    cache: &TranspositionTable,
    mut info: SearchInfo,
    output: &mut W,
) -> Result<Move, SearchError> {
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
            Err(StopReason::TimeUp) => break,
            Err(StopReason::NodesExceeded) => break,
        };

        let elapsed = start.elapsed();
        let nodes_per_second = (stats.nodes as f64 / elapsed.as_secs_f64()) as usize;
        info.accumulated_nodes += stats.nodes;

        let mut score = "score ".to_string();
        if best_eval >= CHECKMATE_MIN {
            score += &format!("mate {}", (depth + 1) / 2);
        } else if best_eval <= -CHECKMATE_MIN {
            score += &format!("mate -{}", depth / 2);
        } else {
            score += &format!("cp {}", best_eval);
        }

        let pv_line = get_pv_line(&mut info, &mut stats, cache, depth)?;
        let pv_string = pv_line
            .iter()
            .map(|mov| mov.to_string())
            .collect::<Vec<String>>()
            .join(" ");
        let info_str = format!(
            "info depth {} {} time {} nodes {} nps {:.2} pv {}",
            depth,
            score,
            elapsed.as_millis(),
            stats.nodes,
            nodes_per_second,
            pv_string,
        );
        // TODO: Remove this unwrap
        writeln!(output, "{}", info_str).unwrap();

        best_move = pv_line.get(0).cloned();

        // If we alreay found a checkmate we dont need to search deeper,
        // as there can only be a checkmate in more moves. But as we already
        // penalize checkmates at a deeper depth, we just can cut here.
        if !info.infinite && best_eval >= CHECKMATE_MIN {
            break;
        }
    }

    match best_move {
        Some(mov) => Ok(mov),
        None => {
            // If there is no best move, choose a random move as we did not
            // have enough time to search the best move.
            let move_generator = MoveGenerator::new(&info.board);
            let mut stats = SearchStats::new(0);

            let moves = move_generator.collect::<Vec<Move>>();
            let mut scored_moves = score_moves(&info, &mut stats, moves, None);
            let next_move = pick_next_move(0, &mut scored_moves);
            Ok(next_move)
        }
    }
}

pub(crate) fn get_pv_line(
    info: &mut SearchInfo,
    stats: &mut SearchStats,
    cache: &TranspositionTable,
    max_depth: u8,
) -> Result<Vec<Move>, MoveGeneratorError> {
    let mut pv = Vec::new();

    let mut board = info.board.clone();
    for _ in 0..max_depth {
        let pv_move = match probe_pv_move(info, stats, cache)? {
            Some(mov) => mov,
            None => break,
        };

        board.make(pv_move);
        pv.push(pv_move);
    }

    Ok(pv)
}

pub(crate) fn probe_pv_move(
    info: &mut SearchInfo,
    stats: &mut SearchStats,
    cache: &TranspositionTable,
) -> Result<Option<Move>, MoveGeneratorError> {
    let entry = match cache.probe(stats, info.board.hash()) {
        Some(entry) => entry,
        None => return Ok(None),
    };

    let best_move = match entry.best_move() {
        Some(mov) => mov,
        None => return Ok(None),
    };

    if !move_exists(&info, best_move)? {
        return Ok(None);
    }

    Ok(Some(best_move))
}

pub(crate) fn move_exists(info: &SearchInfo, given: Move) -> Result<bool, MoveGeneratorError> {
    let move_generator = MoveGenerator::new(&info.board);
    for mov in move_generator {
        if mov == given {
            return Ok(true);
        }
    }

    return Ok(false);
}
