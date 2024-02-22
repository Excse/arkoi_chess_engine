use base::zobrist::ZobristHash;

use super::entry::TranspositionEntry;

#[derive(Clone, Debug, Default)]
pub struct PackedEntry {
    pub(super) key: ZobristHash,
    pub(super) data: u64,
}

impl PackedEntry {
    pub fn pack(_entry: TranspositionEntry) -> Self {
        todo!("Pack TranspositionEntry into PackedEntry")
    }

    pub fn unpack(&self) -> TranspositionEntry {
        todo!("Unpack PackedEntry into TranspositionEntry")
    }
}
