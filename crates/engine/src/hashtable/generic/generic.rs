use base::zobrist::ZobristHash;

const MEGA_BYTE: usize = 1024 * 1024;

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

    pub fn size(size: usize) -> Self {
        let entry_size = std::mem::size_of::<Option<T>>();
        let bytes = size * MEGA_BYTE;

        let entries = bytes / entry_size;
        Self::entries(entries)
    }

    pub fn store(&mut self, key: ZobristHash, entry: T) {
        let index = key.hash() as usize % self.size;

        let stored = &self.entries[index];
        if let Some(stored) = stored {
            if !stored.replaceable(&entry) {
                return;
            }
        }

        self.entries[index] = Some(entry);
    }

    pub fn probe(&self, key: ZobristHash) -> Option<T> {
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
