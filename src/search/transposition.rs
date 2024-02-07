use crate::board::zobrist::ZobristHash;

#[derive(Debug, Clone)]
pub enum TranspositionFlag {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Debug, Clone)]
pub struct TranspositionEntry {
    pub key: ZobristHash,
    pub depth: u8,
    pub flag: TranspositionFlag,
    pub nodes: u64,
    pub eval: isize,
}

impl TranspositionEntry {
    pub fn new(
        key: ZobristHash,
        depth: u8,
        flag: TranspositionFlag,
        nodes: u64,
        eval: isize,
    ) -> Self {
        Self {
            key,
            depth,
            flag,
            nodes,
            eval,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TranspositionTable {
    pub size: usize,
    pub entries: Vec<Option<TranspositionEntry>>,
}

impl TranspositionTable {
    pub fn new(bytes: usize) -> Self {
        let size = bytes / std::mem::size_of::<Option<TranspositionEntry>>();
        let size = to_power_2(size);
        assert!(size > 0);

        let entries = vec![None; size];
        Self { size, entries }
    }

    pub fn probe(&self, key: ZobristHash) -> Option<&TranspositionEntry> {
        let index = key.0 as usize & (self.size - 1);
        match self.entries[index].as_ref() {
            Some(entry) if entry.key == key => Some(entry),
            _ => None,
        }
    }

    pub fn store(&mut self, entry: TranspositionEntry) {
        let index = entry.key.0 as usize & (self.size - 1);
        if let Some(stored) = &self.entries[index] {
            if stored.depth > entry.depth {
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
