use base::zobrist::ZobristHash;

use crate::search::SearchStats;

use super::{
    entry::TranspositionEntry,
    packed::{PackedEntry, NULL_ENTRY},
};

#[derive(Debug)]
pub struct TranspositionTable {
    pub(crate) size: usize,
    pub(crate) entries_ptr: *mut PackedEntry,
}

impl TranspositionTable {
    pub fn entries(entries: usize) -> Self {
        let size = entries.next_power_of_two();
        assert!(size <= isize::MAX as usize);
        assert!(size > 0);

        let mut entries = vec![PackedEntry::default(); size];
        let entries_ptr = entries.as_mut_ptr();
        entries.leak();

        Self { size, entries_ptr }
    }

    pub fn size(size: usize) -> Self {
        let entries = size / std::mem::size_of::<PackedEntry>();
        Self::entries(entries)
    }

    pub fn store(&self, stats: &mut SearchStats, key: ZobristHash, entry: TranspositionEntry) {
        let index = key.hash() as usize % self.size;

        let stored = unsafe { &mut *self.entries_ptr.add(index) };

        // We don't want any replacement schema, thus we just replace
        // every time.
        // TODO: Maybe change this in the future.

        if stored != NULL_ENTRY {
            stats.table.overwrites += 1;
        } else {
            stats.table.new += 1;
        }

        let new_data = PackedEntry::pack(key, entry);
        stored.data = new_data.data;
        stored.key = new_data.key;
    }

    pub fn probe(&self, stats: &mut SearchStats, key: ZobristHash) -> Option<TranspositionEntry> {
        let index = key.hash() as usize % self.size;

        let data = unsafe { &*self.entries_ptr.add(index) };

        let stored_key = data.key ^ data.data;
        if stored_key != key {
            stats.table.misses += 1;
            return None;
        } else {
            stats.table.hits += 1;
        }


        let entry = data.unpack();
        Some(entry)
    }
}

// As we just have a raw pointer to the entries we manually need to drop
// the allocated memory.
impl Drop for TranspositionTable {
    fn drop(&mut self) {
        // SAFE: The HashTable will be dropped only once.
        unsafe {
            let _ = Box::from_raw(self.entries_ptr);
        }
    }
}

// As rustc notices the Transposition Table is not thread-safe. But as we
// want to use LazySMP we need to force this.
unsafe impl Send for TranspositionTable {}
unsafe impl Sync for TranspositionTable {}
