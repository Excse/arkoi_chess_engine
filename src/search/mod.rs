mod error;
mod iterative;
mod killers;
mod negamax;
mod quiescence;
mod sort;

use crate::{
    board::Board,
    generation::mov::Move,
    hashtable::{transposition::TranspositionEntry, HashTable},
    uci::commands::GoCommand,
};

use self::{
    error::{NoDepthOrInfinite, SearchError},
    iterative::iterative_deepening,
};

pub const MAX_DEPTH: u8 = 64;

pub const CHECKMATE: isize = 1_000_000;
pub const CHECKMATE_MIN: isize = CHECKMATE - MAX_DEPTH as isize;
pub const DRAW: isize = 0;

pub const MAX_EVAL: isize = CHECKMATE + 1;
pub const MIN_EVAL: isize = -CHECKMATE - 1;

pub const NULL_DEPTH_REDUCTION: u8 = 2;

pub fn search(
    board: &mut Board,
    cache: &mut HashTable<TranspositionEntry>,
    command: &GoCommand,
) -> Result<Option<Move>, SearchError> {
    let max_depth = match command.depth {
        Some(depth) => depth,
        None => {
            if !command.infinite {
                return Err(NoDepthOrInfinite.into());
            }

            // TODO: For infinite to work we need to have a different
            // thread that can stop the search.
            panic!("Not implemented yet");
        }
    };

    let max_nodes = match command.nodes {
        Some(nodes) => nodes as usize,
        None => std::usize::MAX,
    };

    let moves = if command.search_moves.is_empty() {
        let move_state = board.get_legal_moves()?;
        if move_state.is_checkmate {
            return Ok(None);
        }

        move_state.moves
    } else {
        let mut board = board.clone();
        let moves = board.make_moves(&command.search_moves)?;
        moves
    };

    let best_move =
        iterative_deepening(board, cache, max_depth, max_nodes, moves, command.infinite);
    Ok(best_move)
}
