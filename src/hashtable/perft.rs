use crate::board::zobrist::ZobristHash;

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

    fn replaceable(&self, _: &PerftEntry) -> bool {
        true
    }
}
