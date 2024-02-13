use crate::generation::mov::Move;

use super::{MAX_DEPTH, sort::SCORE_SLICE};

pub const MATE_KILLER_SCORE: usize = SCORE_SLICE * 3;
pub const MATE_KILLER_REDUCTION: usize = 100;

pub const KILLER_SCORE: usize = SCORE_SLICE * 2;
pub const KILLER_REDUCTION: usize = 100;

// We only use two killers, as the benefit does not outweigh the cost
// of iterating and storing more killers.
pub const MAX_KILLERS: usize = 2;

#[derive(Debug)]
pub struct Killers {
    pub moves: [[Option<Move>; MAX_KILLERS]; MAX_DEPTH as usize],
}

impl Default for Killers {
    fn default() -> Self {
        Killers {
            moves: [[None; MAX_KILLERS]; MAX_DEPTH as usize],
        }
    }
}

impl Killers {
    pub fn store(&mut self, mov: &Move, ply: u8) {
        // The current killers from this ply
        let killers = &mut self.moves[ply as usize];
        // We dont want to store the same move twice.
        match &killers[0] {
            Some(killer) if killer == mov => return,
            _ => {
                killers[1] = killers[0];
                killers[0] = Some(*mov);
            }
        }
    }

    pub fn contains(&self, mov: &Move, ply: u8) -> Option<usize> {
        let killers = &self.moves[ply as usize];

        for index in 0..MAX_KILLERS {
            match &killers[index] {
                Some(killer) if killer == mov => return Some(index),
                _ => {}
            }
        }

        None
    }
}
