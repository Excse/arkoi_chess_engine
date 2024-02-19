use crate::{board::zobrist::ZobristHash, generation::mov::Move};

use super::HashEntry;

#[derive(Debug, Clone, Copy)]
pub enum TranspositionFlag {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Debug, Clone)]
pub struct TranspositionEntry {
    key: ZobristHash,
    depth: u8,
    flag: TranspositionFlag,
    eval: isize,
    best_move: Option<Move>,
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

    #[inline(always)]
    pub const fn key(&self) -> ZobristHash {
        self.key
    }

    #[inline(always)]
    pub const fn depth(&self) -> u8 {
        self.depth
    }

    #[inline(always)]
    pub const fn eval(&self) -> isize {
        self.eval
    }

    #[inline(always)]
    pub const fn best_move(&self) -> Option<Move> {
        self.best_move
    }

    #[inline(always)]
    pub fn flag(&self) -> TranspositionFlag {
        self.flag
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
