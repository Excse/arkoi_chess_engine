use std::time::Instant;

use crate::{
    board::Board,
    generation::{error::MoveGeneratorError, mov::Move, MoveGenerator},
    hashtable::{transposition::TranspositionEntry, HashTable},
    search::{negamax::negamax, CHECKMATE_MIN, MAX_EVAL, MIN_EVAL},
};

use super::{
    error::{InCheckmate, SearchError},
    killers::Killers,
    sort::{pick_next_move, score_moves},
    TimeFrame,
};

pub fn iterative_deepening(
    board: &mut Board,
    cache: &mut HashTable<TranspositionEntry>,
    time_frame: &TimeFrame,
    max_depth: u8,
    max_nodes: usize,
    moves: Vec<Move>,
    infinite: bool,
) -> Result<Move, SearchError> {
    if moves.is_empty() {
        return Err(InCheckmate.into());
    }

    let mut best_move = None;

    let mut mate_killers = Killers::default();
    let mut killers = Killers::default();

    let mut accumulated_nodes = 0;

    for depth in 1..=max_depth {
        let start = Instant::now();

        let mut child_nodes = 0;

        // TODO: Use  the given moves
        let result = negamax(
            board,
            cache,
            &mut killers,
            &mut mate_killers,
            &mut child_nodes,
            time_frame,
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

        print!(
            "info depth {} {} time {} nodes {} nps {:.2} pv ",
            depth,
            score,
            elapsed.as_millis(),
            child_nodes,
            nodes_per_second,
        );
        let pv_line = get_pv_line(board, cache, depth)?;
        for mov in &pv_line {
            print!("{} ", mov);
        }
        println!();

        best_move = pv_line.get(0).cloned();

        accumulated_nodes += child_nodes;
        if accumulated_nodes >= max_nodes {
            break;
        }

        // If we alreay found a checkmate we dont need to search deeper,
        // as there can only be a checkmate in more moves. But as we already
        // penalize checkmates at a deeper depth, we just can cut here.
        if !infinite && best_eval >= CHECKMATE_MIN {
            break;
        }
    }

    match best_move {
        Some(mov) => Ok(mov),
        None => {
            // If there is no best move choose a random move as we did not
            // have enough time to search the best move.
            let mut scored_moves = score_moves(moves, 0, None, &killers, &mate_killers);
            let next_move = pick_next_move(0, &mut scored_moves);
            Ok(next_move)
        }
    }
}

fn get_pv_line(
    board: &mut Board,
    cache: &mut HashTable<TranspositionEntry>,
    max_depth: u8,
) -> Result<Vec<Move>, MoveGeneratorError> {
    let mut pv = Vec::new();

    for _ in 0..max_depth {
        let pv_move = match probe_pv_move(board, cache)? {
            Some(mov) => mov,
            None => break,
        };

        board.make(&pv_move);
        pv.push(pv_move);
    }

    for mov in pv.iter().rev() {
        board.unmake(&mov);
    }

    Ok(pv)
}

fn probe_pv_move(
    board: &Board,
    cache: &mut HashTable<TranspositionEntry>,
) -> Result<Option<Move>, MoveGeneratorError> {
    let entry = match cache.probe(board.hash()) {
        Some(entry) => entry,
        None => return Ok(None),
    };

    let best_move = match entry.best_move {
        Some(mov) => mov,
        None => return Ok(None),
    };

    if !move_exists(&board, best_move)? {
        return Ok(None);
    }

    Ok(Some(best_move))
}

fn move_exists(board: &Board, given: Move) -> Result<bool, MoveGeneratorError> {
    let move_generator = MoveGenerator::new(board);
    for mov in move_generator {
        if mov == given {
            return Ok(true);
        }
    }

    return Ok(false);
}
