use base::r#move::Move;

use crate::{evaluation::evaluate, generator::MoveGenerator};

use super::{
    should_stop_search,
    sort::{pick_next_move, score_moves},
    SearchInfo, SearchStats, StopReason, CHECKMATE, CHECKMATE_MIN,
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
pub(crate) fn quiescence(
    info: &mut SearchInfo,
    stats: &mut SearchStats,
    mut alpha: i32,
    beta: i32,
) -> Result<i32, StopReason> {
    stats.nodes += 1;

    should_stop_search(info, stats)?;

    let standing_pat = evaluate(&info.board, info.board.active());

    // If the evaluation exceeds the upper bound we just fail hard.
    if standing_pat >= beta {
        return Ok(beta);
    }

    // Set the new lower bound if the evaluation is better than the current.
    if standing_pat > alpha {
        alpha = standing_pat;
    }

    // TODO: We need to generate only attacking moves.
    let move_generator = MoveGenerator::new(&info.board);
    // TODO: Test if this is useful
    if move_generator.is_checkmate(&info.board) {
        let eval = -CHECKMATE + stats.ply() as i32;
        return Ok(eval);
    }

    // ~~~~~~~~~ MOVE ORDERING ~~~~~~~~~
    // Used to improve the efficiency of the alpha-beta algorithm.
    // Source: https://www.chessprogramming.org/Move_Ordering
    // TODO: Only do capture
    let moves = move_generator.collect::<Vec<Move>>();
    let mut scored_moves = score_moves(info, stats, moves, None);
    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

    for move_index in 0..scored_moves.len() {
        let next_move = pick_next_move(move_index, &mut scored_moves);

        // TODO: This needs to be removed when we can generate only attacking moves.
        if !next_move.is_capture() {
            continue;
        }

        info.board.make(next_move);
        stats.make_search(1);

        let result = quiescence(info, stats, -beta, -alpha);

        info.board.unmake(next_move);
        stats.unmake_search(1);

        let child_eval = -result?;
        alpha = alpha.max(child_eval);

        // If alpha is greater or equal to beta, we need to make
        // a beta cut-off. All other moves will be worse than the
        // current best move.
        if alpha >= beta {
            // We differentiate between mate and normal killers, as mate killers
            // will have a higher score and thus will be prioritized.
            if alpha.abs() >= CHECKMATE_MIN {
                info.mate_killers.store(&next_move, stats.ply());
            } else {
                info.killers.store(&next_move, stats.ply());
            }

            return Ok(beta);
        }
    }

    Ok(alpha)
}
