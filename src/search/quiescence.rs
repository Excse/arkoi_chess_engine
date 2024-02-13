use crate::{board::Board, evaluation::evaluate};

use super::{killers::Killers, sort::sort_moves, CHECKMATE, CHECKMATE_MIN};

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
pub fn quiescence(
    board: &mut Board,
    killers: &mut Killers,
    mate_killers: &mut Killers,
    nodes: &mut usize,
    ply: u8,
    mut alpha: isize,
    beta: isize,
) -> isize {
    *nodes += 1;

    let standing_pat = evaluate(board, board.gamestate.active);

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
        sort_moves(ply, first, second, &pv_move, killers, mate_killers)
    });
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    for mov in move_state.moves {
        // TODO: This needs to be removed when we can generate only attacking moves.
        if !mov.is_capture() {
            continue;
        }

        board.make(&mov);

        let child_eval = -quiescence(board, killers, mate_killers, nodes, ply + 1, -beta, -alpha);
        alpha = alpha.max(child_eval);

        board.unmake(&mov);

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
