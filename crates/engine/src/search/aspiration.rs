use crate::hashtable::TranspositionTable;

use super::{communication::SearchSender, negamax::negamax, SearchInfo, SearchStats, StopReason};

pub const ASPIRATION_WINDOW: i32 = 20;

pub fn aspiration<S: SearchSender>(
    cache: &TranspositionTable,
    info: &mut SearchInfo<S>,
    stats: &mut SearchStats,
    last_eval: i32,
) -> Result<i32, StopReason> {
    let mut window = ASPIRATION_WINDOW;
    let mut eval = last_eval;

    loop {
        let alpha = eval - window;
        let beta = eval + window;

        eval = negamax(cache, info, stats, alpha, beta, false, false)?;

        if alpha < eval && eval < beta {
            break;
        }

        window *= 2;
    }

    Ok(eval)
}
