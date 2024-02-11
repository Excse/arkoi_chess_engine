use crate::board::zobrist::ZobristHash;

use super::HashEntry;

#[derive(Debug, Clone)]
pub enum TranspositionFlag {
    Exact,
    Alpha,
    Beta,
}

#[derive(Debug, Clone)]
pub struct TranspositionEntry {
    pub key: ZobristHash,
    pub depth: u8,
    pub flag: TranspositionFlag,
    pub eval: isize,
}

impl TranspositionEntry {
    pub fn new(key: ZobristHash, depth: u8, flag: TranspositionFlag, eval: isize) -> Self {
        Self {
            key,
            depth,
            flag,
            eval,
        }
    }
}

impl HashEntry<TranspositionEntry> for TranspositionEntry {
    fn key(&self) -> ZobristHash {
        self.key
    }

    fn replaceable(&self, other: &TranspositionEntry) -> bool {
        self.depth < other.depth
    }
}
