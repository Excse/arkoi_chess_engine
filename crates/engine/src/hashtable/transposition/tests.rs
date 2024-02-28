#[cfg(test)]
mod table {
    use base::zobrist::ZobristHash;

    use crate::{
        hashtable::{
            entry::{TranspositionEntry, TranspositionFlag},
            TranspositionTable,
        },
        search::SearchStats,
    };

    #[test]
    fn right_size_1() {
        let table = TranspositionTable::size(1024);
        assert_eq!(table.size, 64);
        assert_ne!(table.entries_ptr, std::ptr::null_mut());
    }

    #[test]
    fn right_size_2() {
        let table = TranspositionTable::size(777);
        assert_eq!(table.size, 64);
        assert_ne!(table.entries_ptr, std::ptr::null_mut());
    }

    #[test]
    fn right_size_3() {
        let table = TranspositionTable::size(67108864);
        assert_eq!(table.size, 4194304);
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
        let mut stats = SearchStats::new(42);

        let stored_entry = TranspositionEntry::new(42, TranspositionFlag::UpperBound, 42, None);
        let stored_key = ZobristHash::new(0x4242424242424242);
        table.store(&mut stats, stored_key, stored_entry.clone());

        assert_eq!(stats.table.new, 1);
        assert_eq!(stats.table.overwrites, 0);
        assert_eq!(stats.table.misses, 0);
        assert_eq!(stats.table.hits, 0);

        let probed_entry = table.probe(&mut stats, stored_key);
        assert_eq!(Some(stored_entry.clone()), probed_entry);
        assert_eq!(stats.table.new, 1);
        assert_eq!(stats.table.overwrites, 0);
        assert_eq!(stats.table.misses, 0);
        assert_eq!(stats.table.hits, 1);
    }
}
