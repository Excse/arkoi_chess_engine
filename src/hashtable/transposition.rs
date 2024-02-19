use crate::{board::zobrist::ZobristHash, generation::mov::Move};

use super::HashEntry;

#[derive(Debug, Clone)]
pub enum TranspositionFlag {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Debug, Clone)]
pub struct TranspositionEntry {
    pub key: ZobristHash,
    pub depth: u8,
    pub flag: TranspositionFlag,
    pub eval: isize,
    pub best_move: Option<Move>,
}

impl TranspositionEntry {
    pub fn new(
        key: ZobristHash,
        depth: u8,
        flag: TranspositionFlag,
        eval: isize,
        best_move: Option<Move>,
    ) -> Self {
        Self {
            key,
            depth,
            flag,
            eval,
            best_move,
        }
    }
}

impl HashEntry<TranspositionEntry> for TranspositionEntry {
    fn key(&self) -> ZobristHash {
        self.key
    }

    fn replaceable(&self, _: &TranspositionEntry) -> bool {
        true
    }
}
