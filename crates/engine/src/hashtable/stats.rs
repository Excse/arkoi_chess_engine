#[derive(Debug, Default)]
pub struct HashTableStats {
    pub hits: u64,
    pub misses: u64,
    pub overwrites: u64,
    pub new: u64,
}

impl HashTableStats {
    pub fn new() -> Self {
        Self {
            hits: 0,
            misses: 0,
            overwrites: 0,
            new: 0,
        }
    }
}
