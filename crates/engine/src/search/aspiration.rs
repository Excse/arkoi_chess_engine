use crate::hashtable::TranspositionTable;

use super::{
    communication::SearchSender, negamax::negamax, SearchInfo, SearchStats, StopReason, MAX_EVAL,
    MIN_EVAL,
};

pub const ASPIRATION_WINDOW: i32 = 20;

pub fn aspiration<S: SearchSender>(
    cache: &TranspositionTable,
    info: &mut SearchInfo<S>,
    stats: &mut SearchStats,
    last_eval: i32,
) -> Result<i32, StopReason> {
    let alpha = last_eval - ASPIRATION_WINDOW;
    let beta = last_eval + ASPIRATION_WINDOW;

    let eval = negamax(cache, info, stats, alpha, beta, false, false, false)?;
    if eval <= alpha || eval >= beta {
        return negamax(cache, info, stats, MIN_EVAL, MAX_EVAL, false, false, false);
    }

    Ok(eval)
}
