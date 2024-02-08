use crate::{board::zobrist::ZobristHash, perft::PerftStats};

use super::HashEntry;

#[derive(Debug, Clone)]
pub struct PerftEntry {
    pub key: ZobristHash,
    pub depth: u8,
    pub nodes: u64,
}

impl PerftEntry {
    pub fn new(key: ZobristHash, depth: u8, nodes: u64) -> Self {
        Self { key, depth, nodes }
    }
}

impl HashEntry<PerftEntry> for PerftEntry {
    fn key(&self) -> ZobristHash {
        self.key
    }

    fn replaceable(&self, other: &PerftEntry) -> bool {
        self.depth < other.depth
    }
}

#[derive(Debug, Clone)]
pub struct PerftStatsEntry {
    pub key: ZobristHash,
    pub depth: u8,
    pub stats: PerftStats,
}

impl PerftStatsEntry {
    pub fn new(key: ZobristHash, depth: u8, stats: PerftStats) -> Self {
        Self { key, depth, stats }
    }
}

impl HashEntry<PerftStatsEntry> for PerftStatsEntry {
    fn key(&self) -> ZobristHash {
        self.key
    }

    fn replaceable(&self, other: &PerftStatsEntry) -> bool {
        self.depth < other.depth
    }
}
