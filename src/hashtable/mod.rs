use crate::board::zobrist::ZobristHash;

pub mod perft;
pub mod transposition;

pub trait HashEntry<T> {
    fn key(&self) -> ZobristHash;

    fn replaceable(&self, other: &T) -> bool;
}

#[derive(Debug, Default)]
pub struct HashTableStats {
    pub hits: usize,
    pub misses: usize,
    pub new: usize,
    pub overwrites: usize,
}

#[derive(Debug)]
pub struct HashTable<T: Clone + HashEntry<T>> {
    pub size: usize,
    pub entries: Vec<Option<T>>,
    pub stats: HashTableStats,
}

impl<T: Clone + HashEntry<T>> HashTable<T> {
    pub fn entries(entries: usize) -> Self {
        let size = Self::to_power_2(entries);
        assert!(size > 0);

        let entries = vec![None; size];
        Self {
            size,
            entries,
            stats: HashTableStats::default(),
        }
    }

    pub fn size(size: usize) -> Self {
        let entries = size / std::mem::size_of::<Option<T>>();
        Self::entries(entries)
    }

    pub fn probe(&mut self, key: ZobristHash) -> Option<&T> {
        let index = key.0 as usize & (self.size - 1);
        match self.entries[index].as_ref() {
            Some(entry) if entry.key() == key => {
                self.stats.hits += 1;
                Some(entry)
            }
            _ => {
                self.stats.misses += 1;
                None
            }
        }
    }

    pub fn store(&mut self, entry: T) {
        let index = entry.key().0 as usize & (self.size - 1);
        if let Some(stored) = &self.entries[index] {
            if !stored.replaceable(&entry) {
                return;
            }

            self.stats.overwrites += 1;
        } else {
            self.stats.new += 1;
        }

        self.entries[index] = Some(entry);
    }

    pub fn reset_stats(&mut self) {
        self.stats = HashTableStats::default();
    }

    fn is_power_2(value: usize) -> bool {
        if value == 0 {
            return false;
        }

        let result = value & (value - 1);
        result == 0
    }

    fn to_power_2(mut value: usize) -> usize {
        if !Self::is_power_2(value) {
            let mut power = 1;
            while power < value {
                power <<= 1;
            }

            value = power;
        }

        value
    }
}
