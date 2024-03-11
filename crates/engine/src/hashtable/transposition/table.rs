use base::zobrist::ZobristHash;

use super::{
    entry::TranspositionEntry,
    packed::{PackedEntry, NULL_ENTRY},
};

const MEGA_BYTE: usize = 1024 * 1024;

#[derive(Debug)]
pub struct TranspositionTable {
    pub(crate) capacity: usize,
    pub(crate) entries_ptr: *mut PackedEntry,

    // Some stats about the table.
    pub(crate) inserted_ptr: *mut usize,
    pub(crate) misses_ptr: *mut usize,
    pub(crate) hits_ptr: *mut usize,
    pub(crate) overwrites_ptr: *mut usize,
    pub(crate) age_ptr: *mut u64,
}

impl TranspositionTable {
    pub fn entries(entries: usize) -> Self {
        let capacity = entries.next_power_of_two();
        assert!(capacity <= isize::MAX as usize);
        assert!(capacity > 0);

        let mut entries = vec![PackedEntry::default(); capacity];
        let entries_ptr = entries.as_mut_ptr();
        entries.leak();

        let inserted = Box::new(0);
        let inserted_ptr = Box::into_raw(inserted);

        let misses = Box::new(0);
        let misses_ptr = Box::into_raw(misses);

        let hits = Box::new(0);
        let hits_ptr = Box::into_raw(hits);

        let overwrites = Box::new(0);
        let overwrites_ptr = Box::into_raw(overwrites);

        let age = Box::new(0);
        let age_ptr = Box::into_raw(age);

        Self {
            capacity,
            entries_ptr,
            inserted_ptr,
            misses_ptr,
            hits_ptr,
            overwrites_ptr,
            age_ptr,
        }
    }

    pub fn size(size: usize) -> Self {
        let entry_size = std::mem::size_of::<PackedEntry>();
        let bytes = size * MEGA_BYTE;

        let entries = bytes / entry_size;
        Self::entries(entries)
    }

    pub fn store(&self, key: ZobristHash, entry: TranspositionEntry) {
        let index = key.hash() as usize & (self.capacity - 1);

        let stored = unsafe { &mut *self.entries_ptr.add(index) };
        let age = unsafe { *self.age_ptr };

        if age <= stored.age {
            let stored_entry = stored.unpack();
            if stored_entry.depth > entry.depth {
                return;
            }
        }

        unsafe {
            if stored != NULL_ENTRY {
                *self.overwrites_ptr += 1;
            } else {
                *self.inserted_ptr += 1;
            }
        }

        let new_data = PackedEntry::pack(key, entry, age);
        stored.data = new_data.data;
        stored.key = new_data.key;
    }

    pub fn probe(&self, key: ZobristHash) -> Option<TranspositionEntry> {
        let index = key.hash() as usize & (self.capacity - 1);

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

    pub fn clear(&self) {
        for index in 0..self.capacity {
            let entry = unsafe { &mut *self.entries_ptr.add(index) };
            *entry = NULL_ENTRY;
        }

        self.reset_stats();
    }

    pub fn reset_stats(&self) {
        unsafe {
            *self.overwrites_ptr = 0;
            *self.inserted_ptr = 0;
            *self.misses_ptr = 0;
            *self.hits_ptr = 0;
        }
    }

    pub fn full_percentage(&self) -> u16 {
        let min_size = self.capacity.min(1000);

        let mut occupied = 0;
        for index in 0..min_size {
            let entry = unsafe { &*self.entries_ptr.add(index) };
            if entry != &NULL_ENTRY {
                occupied += 1;
            }
        }

        let permille = occupied as f64 / min_size as f64;
        (permille * 1000.0) as u16
    }

    pub fn increment_age(&self) {
        unsafe {
            *self.age_ptr += 1;
        }
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
    pub fn inserted(&self) -> usize {
        unsafe { *self.inserted_ptr }
    }

    #[inline(always)]
    pub fn stores(&self) -> usize {
        self.inserted() + self.overwrites()
    }

    #[inline(always)]
    pub fn probes(&self) -> usize {
        self.hits() + self.misses()
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

// As we just have a raw pointer to the entries we manually need to drop
// the allocated memory.
impl Drop for TranspositionTable {
    fn drop(&mut self) {
        // SAFE: The HashTable will be dropped only once.
        unsafe {
            let _ = Box::from_raw(self.entries_ptr);
            let _ = Box::from_raw(self.inserted_ptr);
            let _ = Box::from_raw(self.misses_ptr);
            let _ = Box::from_raw(self.hits_ptr);
            let _ = Box::from_raw(self.overwrites_ptr);
            let _ = Box::from_raw(self.age_ptr);
        }
    }
}

// As rustc notices the Transposition Table is not thread-safe. But as we
// want to use LazySMP we need to force this.
unsafe impl Send for TranspositionTable {}
unsafe impl Sync for TranspositionTable {}
