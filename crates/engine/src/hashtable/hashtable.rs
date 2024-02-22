use base::zobrist::ZobristHash;

pub trait HashTable<T> {
    fn store(&mut self, key: ZobristHash, entry: T);

    fn probe(&self, key: ZobristHash) -> Option<T>;
}
