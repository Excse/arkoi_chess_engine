use base::zobrist::ZobristHash;

use super::{
    entry::TranspositionEntry,
    packed::{PackedEntry, NULL_ENTRY},
};

#[derive(Debug)]
pub struct TranspositionTable {
    pub(crate) size: usize,
    pub(crate) entries_ptr: *mut PackedEntry,

    // Some stats about the table.
    pub(crate) occupied_ptr: *mut usize,
    pub(crate) misses_ptr: *mut usize,
    pub(crate) hits_ptr: *mut usize,
    pub(crate) overwrites_ptr: *mut usize,
}

impl TranspositionTable {
    pub fn entries(entries: usize) -> Self {
        let size = entries.next_power_of_two();
        assert!(size <= isize::MAX as usize);
        assert!(size > 0);

        let mut entries = vec![PackedEntry::default(); size];
        let entries_ptr = entries.as_mut_ptr();
        entries.leak();

        let occupied = Box::new(0);
        let occupied_ptr = Box::into_raw(occupied);

        let misses = Box::new(0);
        let misses_ptr = Box::into_raw(misses);

        let hits = Box::new(0);
        let hits_ptr = Box::into_raw(hits);

        let overwrites = Box::new(0);
        let overwrites_ptr = Box::into_raw(overwrites);

        Self {
            size,
            entries_ptr,
            occupied_ptr,
            misses_ptr,
            hits_ptr,
            overwrites_ptr,
        }
    }

    pub fn size(size: usize) -> Self {
        let entries = size / std::mem::size_of::<PackedEntry>();
        Self::entries(entries)
    }

    pub fn store(&self, key: ZobristHash, entry: TranspositionEntry) {
        let index = key.hash() as usize % self.size;

        let stored = unsafe { &mut *self.entries_ptr.add(index) };

        // We don't want any replacement schema, thus we just replace
        // every time.
        // TODO: Maybe change this in the future.

        unsafe {
            if stored != NULL_ENTRY {
                *self.overwrites_ptr += 1;
            } else {
                *self.occupied_ptr += 1;
            }
        }

        let new_data = PackedEntry::pack(key, entry);
        stored.data = new_data.data;
        stored.key = new_data.key;
    }

    pub fn probe(&self, key: ZobristHash) -> Option<TranspositionEntry> {
        let index = key.hash() as usize % self.size;

        let data = unsafe { &*self.entries_ptr.add(index) };

        let stored_key = data.key ^ data.data;
        unsafe {
            if stored_key != key {
                *self.misses_ptr += 1;
                return None;
            } else {
                *self.hits_ptr += 1;
            }
        }

        let entry = data.unpack();
        Some(entry)
    }

    pub fn full_percentage(&self) -> u8 {
        let occupied = unsafe { *self.occupied_ptr };
        let size = self.size as f64;
        let permill = (occupied as f64 / size as f64) * 1000.0;
        permill as u8
    }

    #[inline(always)]
    pub fn hits(&self) -> usize {
        unsafe { *self.hits_ptr }
    }

    #[inline(always)]
    pub fn misses(&self) -> usize {
        unsafe { *self.misses_ptr }
    }

    #[inline(always)]
    pub fn overwrites(&self) -> usize {
        unsafe { *self.overwrites_ptr }
    }

    #[inline(always)]
    pub fn occupied(&self) -> usize {
        unsafe { *self.occupied_ptr }
    }
}

// As we just have a raw pointer to the entries we manually need to drop
// the allocated memory.
impl Drop for TranspositionTable {
    fn drop(&mut self) {
        // SAFE: The HashTable will be dropped only once.
        unsafe {
            let _ = Box::from_raw(self.entries_ptr);
            let _ = Box::from_raw(self.occupied_ptr);
            let _ = Box::from_raw(self.misses_ptr);
            let _ = Box::from_raw(self.hits_ptr);
            let _ = Box::from_raw(self.overwrites_ptr);
        }
    }
}

// As rustc notices the Transposition Table is not thread-safe. But as we
// want to use LazySMP we need to force this.
unsafe impl Send for TranspositionTable {}
unsafe impl Sync for TranspositionTable {}
