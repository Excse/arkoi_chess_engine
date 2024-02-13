use crate::{
    board::Board,
    hashtable::{transposition::TranspositionEntry, HashTable},
    generation::mov::Move,
    search::{negamax::negamax, CHECKMATE_MIN, MAX_EVAL, MIN_EVAL},
};

use super::killers::Killers;

pub fn iterative_deepening(
    board: &mut Board,
    cache: &mut HashTable<TranspositionEntry>,
    max_depth: u8,
    max_nodes: usize,
    _moves: Vec<Move>,
    infinite: bool,
) -> Option<Move> {
    let mut best_move = None;

    let mut mate_killers = Killers::default();
    let mut killers = Killers::default();

    let mut accumulated_nodes = 0;
    let mut last_nodes = 0;

    let mut parent_pv = Vec::new();
    for depth in 1..=max_depth {
        let start = std::time::Instant::now();

        let mut child_nodes = 0;

        // TODO: Use  the given moves
        let best_eval = negamax(
            board,
            cache,
            &mut parent_pv,
            &mut killers,
            &mut mate_killers,
            &mut child_nodes,
            depth,
            0,
            MIN_EVAL,
            MAX_EVAL,
            false,
            false,
        );

        let elapsed = start.elapsed();

        let nodes_per_second = (child_nodes as f64 / elapsed.as_secs_f64()) as usize;
        let branch_factor = child_nodes as f64 / last_nodes as f64;

        let mut score = "score ".to_string();
        if best_eval >= CHECKMATE_MIN {
            score += &format!("mate {}", (depth + 1) / 2);
        } else if best_eval <= -CHECKMATE_MIN {
            score += &format!("mate -{}", depth / 2);
        } else {
            score += &format!("cp {}", best_eval);
        }

        println!(
            "info depth {} {} time {} nodes {} nps {:.2} pv {}",
            depth,
            score,
            elapsed.as_millis(),
            child_nodes,
            nodes_per_second,
            parent_pv
                .iter()
                .map(|mov| mov.to_string())
                .collect::<Vec<String>>()
                .join(" "),
        );
        println!("Branch factor: {:.2}", branch_factor);

        best_move = parent_pv.first().cloned();
        accumulated_nodes += child_nodes;
        last_nodes = child_nodes;

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

    best_move
}
