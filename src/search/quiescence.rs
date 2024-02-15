use crate::{board::Board, evaluation::evaluate};

use super::{
    error::SearchError,
    killers::Killers,
    sort::{pick_next_move, score_moves},
    TimeFrame, CHECKMATE, CHECKMATE_MIN,
};

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
    time_frame: &TimeFrame,
    ply: u8,
    mut alpha: isize,
    beta: isize,
) -> Result<isize, SearchError> {
    *nodes += 1;

    time_frame.is_time_up()?;

    let standing_pat = evaluate(board, board.gamestate.active);

    // If the evaluation exceeds the upper bound we just fail hard.
    if standing_pat >= beta {
        return Ok(beta);
    }

    // Set the new lower bound if the evaluation is better than the current.
    if standing_pat > alpha {
        alpha = standing_pat;
    }

    // TODO: We need to generate only attacking moves.
    let move_state = board.get_legal_moves().unwrap();
    // TODO: Test if this is useful
    if move_state.is_checkmate {
        let eval = -CHECKMATE + ply as isize;
        return Ok(eval);
    }

    // ~~~~~~~~~ MOVE ORDERING ~~~~~~~~~
    // Used to improve the efficiency of the alpha-beta algorithm.
    // Source: https://www.chessprogramming.org/Move_Ordering
    // TODO: Only do capture
    let mut scored_moves = score_moves(move_state.moves, ply, None, killers, mate_killers);
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    for move_index in 0..scored_moves.len() {
        let next_move = pick_next_move(move_index, &mut scored_moves);

        // TODO: This needs to be removed when we can generate only attacking moves.
        if !next_move.is_capture() {
            continue;
        }

        board.make(&next_move);

        let result = quiescence(
            board,
            killers,
            mate_killers,
            nodes,
            time_frame,
            ply + 1,
            -beta,
            -alpha,
        );
        let child_eval = match result {
            Ok(eval) => -eval,
            Err(error) => {
                board.unmake(&next_move);
                return Err(error);
            }
        };

        alpha = alpha.max(child_eval);

        board.unmake(&next_move);

        // If alpha is greater or equal to beta, we need to make
        // a beta cut-off. All other moves will be worse than the
        // current best move.
        if alpha >= beta {
            // We differentiate between mate and normal killers, as mate killers
            // will have a higher score and thus will be prioritized.
            if alpha.abs() >= CHECKMATE_MIN {
                mate_killers.store(&next_move, ply);
            } else {
                killers.store(&next_move, ply);
            }

            return Ok(beta);
        }
    }

    Ok(alpha)
}
