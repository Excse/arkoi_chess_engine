use base::zobrist::ZobristHash;

use crate::hashtable::HashTable;

pub trait HashEntry<T> {
    fn key(&self) -> ZobristHash;

    fn replaceable(&self, other: &T) -> bool;
}

pub struct GenericTable<T: Clone + HashEntry<T>> {
    size: usize,
    entries: Vec<Option<T>>,
}

impl<T: Clone + HashEntry<T>> GenericTable<T> {
    pub fn entries(entries: usize) -> Self {
        let size = entries.next_power_of_two();
        assert!(size > 0);

        let entries = vec![None; size];
        Self { size, entries }
    }
}

impl<T: Clone + HashEntry<T>> HashTable<T> for GenericTable<T> {
    fn store(&mut self, key: ZobristHash, entry: T) {
        let index = key.hash() as usize % self.size;

        let stored = &self.entries[index];
        if let Some(stored) = stored {
            if !stored.replaceable(&entry) {
                return;
            }
        }

        self.entries[index] = Some(entry);
    }

    fn probe(&self, key: ZobristHash) -> Option<T> {
        let index = key.hash() as usize % self.size;

        let entry = &self.entries[index];
        if let Some(stored) = entry {
            if stored.key() == key {
                return Some(stored.clone());
            }
        }

        None
    }
}
