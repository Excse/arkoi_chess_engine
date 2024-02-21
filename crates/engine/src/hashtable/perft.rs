use api::zobrist::ZobristHash;

use crate::perft::PerftStats;

use super::HashEntry;

#[derive(Debug, Clone)]
pub struct PerftEntry {
    key: ZobristHash,
    depth: u8,
    nodes: u64,
}

impl PerftEntry {
    pub fn new(key: ZobristHash, depth: u8, nodes: u64) -> Self {
        Self { key, depth, nodes }
    }

    #[inline(always)]
    pub const fn nodes(&self) -> u64 {
        self.nodes
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
    key: ZobristHash,
    depth: u8,
    stats: PerftStats,
}

impl PerftStatsEntry {
    pub fn new(key: ZobristHash, depth: u8, stats: PerftStats) -> Self {
        Self { key, depth, stats }
    }

    #[inline(always)]
    pub const fn stats(&self) -> &PerftStats {
        &self.stats
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
