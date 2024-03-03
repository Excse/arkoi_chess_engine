#[cfg(test)]
mod table {
    use base::zobrist::ZobristHash;

    use crate::hashtable::{
        entry::{TranspositionEntry, TranspositionFlag},
        TranspositionTable,
    };

    #[test]
    fn right_size_1() {
        let table = TranspositionTable::size(64);
        assert_eq!(table.size, 64);
        assert_ne!(table.entries_ptr, std::ptr::null_mut());
    }

    #[test]
    fn right_size_2() {
        let table = TranspositionTable::size(22);
        assert_eq!(table.size, 64);
        assert_ne!(table.entries_ptr, std::ptr::null_mut());
    }

    #[test]
    fn right_size_3() {
        let table = TranspositionTable::size(65);
        assert_eq!(table.size, 134217728);
        assert_ne!(table.entries_ptr, std::ptr::null_mut());
    }

    #[test]
    fn right_size_4() {
        let table = TranspositionTable::entries(42);
        assert_eq!(table.size, 64);
        assert_ne!(table.entries_ptr, std::ptr::null_mut());
    }

    #[test]
    fn can_store_1() {
        let table = TranspositionTable::entries(1024);

        let stored_entry = TranspositionEntry::new(42, TranspositionFlag::UpperBound, 42, None);
        let stored_key = ZobristHash::new(0x4242424242424242);
        table.store(stored_key, stored_entry.clone());

        assert_eq!(table.occupied(), 1);
        assert_eq!(table.overwrites(), 0);
        assert_eq!(table.misses(), 0);
        assert_eq!(table.hits(), 0);

        let probed_entry = table.probe(stored_key);
        assert_eq!(Some(stored_entry.clone()), probed_entry);
        assert_eq!(table.occupied(), 1);
        assert_eq!(table.overwrites(), 0);
        assert_eq!(table.misses(), 0);
        assert_eq!(table.hits(), 1);
    }
}
