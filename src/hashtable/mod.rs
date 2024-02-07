use crate::board::zobrist::ZobristHash;

pub mod perft;
pub mod transposition;

pub trait HashEntry<T> {
    fn key(&self) -> ZobristHash;

    fn replaceable(&self, other: &T) -> bool;
}

#[derive(Debug, Clone)]
pub struct HashTable<T: Clone + HashEntry<T>> {
    pub size: usize,
    pub entries: Vec<Option<T>>,
}

impl<T: Clone + HashEntry<T>> HashTable<T> {
    pub fn new(bytes: usize) -> Self {
        let size = bytes / std::mem::size_of::<Option<T>>();
        let size = to_power_2(size);
        assert!(size > 0);

        let entries = vec![None; size];
        Self { size, entries }
    }

    pub fn probe(&self, key: ZobristHash) -> Option<&T> {
        let index = key.0 as usize & (self.size - 1);
        match self.entries[index].as_ref() {
            Some(entry) if entry.key() == key => Some(entry),
            _ => None,
        }
    }

    pub fn store(&mut self, entry: T) {
        let index = entry.key().0 as usize & (self.size - 1);
        if let Some(stored) = &self.entries[index] {
            if !stored.replaceable(&entry) {
                return;
            }
        }

        self.entries[index] = Some(entry);
    }
}

fn is_power_2(value: usize) -> bool {
    if value == 0 {
        return false;
    }

    let result = value & (value - 1);
    result == 0
}

fn to_power_2(mut value: usize) -> usize {
    if !is_power_2(value) {
        let mut power = 1;
        while power < value {
            power <<= 1;
        }

        value = power;
    }

    value
}