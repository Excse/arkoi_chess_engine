use std::{fmt::Write, time::Instant};

use base::{board::Board, r#move::Move};

use crate::{
    generator::{error::MoveGeneratorError, MoveGenerator},
    hashtable::{HashTable, TranspositionTable},
    search::{negamax::negamax, CHECKMATE_MIN, MAX_EVAL, MIN_EVAL},
};

use super::{
    error::SearchError,
    killers::Killers,
    sort::{pick_next_move, score_moves},
    SearchInfo,
};

pub(crate) fn iterative_deepening<W: Write>(
    board: &mut Board,
    cache: &mut TranspositionTable,
    search_info: SearchInfo,
    output: &mut W,
) -> Result<Move, SearchError> {
    let mut best_move = None;

    let mut mate_killers = Killers::default();
    let mut killers = Killers::default();

    let mut accumulated_nodes = 0;

    for depth in 1..=search_info.max_depth() {
        let start = Instant::now();

        let mut child_nodes = 0;

        // TODO: Usethe given moves: info.moves()
        let result = negamax(
            board,
            cache,
            &mut killers,
            &mut mate_killers,
            &mut child_nodes,
            search_info.time_frame(),
            depth,
            0,
            MIN_EVAL,
            MAX_EVAL,
            false,
            false,
        );
        let best_eval = match result {
            Ok(result) => result,
            Err(SearchError::TimeUp(_)) => {
                break;
            }
            Err(error) => return Err(error),
        };

        let elapsed = start.elapsed();
        let nodes_per_second = (child_nodes as f64 / elapsed.as_secs_f64()) as usize;

        let mut score = "score ".to_string();
        if best_eval >= CHECKMATE_MIN {
            score += &format!("mate {}", (depth + 1) / 2);
        } else if best_eval <= -CHECKMATE_MIN {
            score += &format!("mate -{}", depth / 2);
        } else {
            score += &format!("cp {}", best_eval);
        }

        let pv_line = get_pv_line(board, cache, depth)?;
        let pv_string = pv_line
            .iter()
            .map(|mov| mov.to_string())
            .collect::<Vec<String>>()
            .join(" ");
        let info = format!(
            "info depth {} {} time {} nodes {} nps {:.2} pv {}",
            depth,
            score,
            elapsed.as_millis(),
            child_nodes,
            nodes_per_second,
            pv_string,
        );
        // TODO: Remove this unwrap
        writeln!(output, "{}", info).unwrap();

        best_move = pv_line.get(0).cloned();

        accumulated_nodes += child_nodes;
        if accumulated_nodes >= search_info.max_nodes() {
            break;
        }

        // If we alreay found a checkmate we dont need to search deeper,
        // as there can only be a checkmate in more moves. But as we already
        // penalize checkmates at a deeper depth, we just can cut here.
        if !search_info.is_infinite() && best_eval >= CHECKMATE_MIN {
            break;
        }
    }

    match best_move {
        Some(mov) => Ok(mov),
        None => {
            // If there is no best move, choose a random move as we did not
            // have enough time to search the best move.
            let move_generator = MoveGenerator::new(board);
            let moves = move_generator.collect::<Vec<Move>>();
            let mut scored_moves = score_moves(moves, 0, None, &killers, &mate_killers);
            let next_move = pick_next_move(0, &mut scored_moves);
            Ok(next_move)
        }
    }
}

pub(crate) fn get_pv_line(
    board: &mut Board,
    cache: &TranspositionTable,
    max_depth: u8,
) -> Result<Vec<Move>, MoveGeneratorError> {
    let mut pv = Vec::new();

    for _ in 0..max_depth {
        let pv_move = match probe_pv_move(board, cache)? {
            Some(mov) => mov,
            None => break,
        };

        board.make(pv_move);
        pv.push(pv_move);
    }

    for mov in pv.iter().rev() {
        board.unmake(*mov);
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

    if !move_exists(&board, best_move)? {
        return Ok(None);
    }

    Ok(Some(best_move))
}

pub(crate) fn move_exists(board: &Board, given: Move) -> Result<bool, MoveGeneratorError> {
    let move_generator = MoveGenerator::new(board);
    for mov in move_generator {
        if mov == given {
            return Ok(true);
        }
    }

    return Ok(false);
}
