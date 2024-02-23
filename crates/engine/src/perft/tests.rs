#[cfg(test)]
mod perft {
    use std::{fs::File, io::Read};

    use base::{board::Board, zobrist::ZobristHasher};
    use rand::{rngs::StdRng, SeedableRng};

    use crate::{
        hashtable::GenericTable,
        perft::{perft_normal, perft_stats},
    };

    #[test]
    fn perft_startpos_0() {
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let mut cache = GenericTable::size(67108864);

        let mut board = Board::default(hasher);
        let stats = perft_stats::<true>(&mut board, &mut cache, 0);

        assert_eq!(stats.nodes(), 1);
        assert_eq!(stats.captures(), 0);
        assert_eq!(stats.en_passants(), 0);
        assert_eq!(stats.castles(), 0);
        assert_eq!(stats.promotions(), 0);
    }

    #[test]
    fn perft_startpos_1() {
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let mut cache = GenericTable::size(67108864);

        let mut board = Board::default(hasher);
        let stats = perft_stats::<true>(&mut board, &mut cache, 1);

        assert_eq!(stats.nodes(), 20);
        assert_eq!(stats.captures(), 0);
        assert_eq!(stats.en_passants(), 0);
        assert_eq!(stats.castles(), 0);
        assert_eq!(stats.promotions(), 0);
    }

    #[test]
    fn perft_startpos_2() {
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let mut cache = GenericTable::size(67108864);

        let mut board = Board::default(hasher);
        let stats = perft_stats::<true>(&mut board, &mut cache, 2);

        assert_eq!(stats.nodes(), 400);
        assert_eq!(stats.captures(), 0);
        assert_eq!(stats.en_passants(), 0);
        assert_eq!(stats.castles(), 0);
        assert_eq!(stats.promotions(), 0);
    }

    #[test]
    fn perft_startpos_3() {
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let mut cache = GenericTable::size(67108864);

        let mut board = Board::default(hasher);
        let stats = perft_stats::<true>(&mut board, &mut cache, 3);

        assert_eq!(stats.nodes(), 8902);
        assert_eq!(stats.captures(), 34);
        assert_eq!(stats.en_passants(), 0);
        assert_eq!(stats.castles(), 0);
        assert_eq!(stats.promotions(), 0);
    }

    #[test]
    fn perft_startpos_4() {
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let mut cache = GenericTable::size(67108864);

        let mut board = Board::default(hasher);
        let stats = perft_stats::<true>(&mut board, &mut cache, 4);

        assert_eq!(stats.nodes(), 197281);
        assert_eq!(stats.captures(), 1576);
        assert_eq!(stats.en_passants(), 0);
        assert_eq!(stats.castles(), 0);
        assert_eq!(stats.promotions(), 0);
    }

    #[test]
    fn perft_startpos_5() {
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let mut cache = GenericTable::size(67108864);

        let mut board = Board::default(hasher);
        let stats = perft_stats::<true>(&mut board, &mut cache, 5);

        assert_eq!(stats.nodes(), 4865609);
        assert_eq!(stats.captures(), 82719);
        assert_eq!(stats.en_passants(), 258);
        assert_eq!(stats.castles(), 0);
        assert_eq!(stats.promotions(), 0);
    }

    #[test]
    fn perft_startpos_6() {
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let mut cache = GenericTable::size(67108864);

        let mut board = Board::default(hasher);
        let stats = perft_stats::<true>(&mut board, &mut cache, 6);

        assert_eq!(stats.nodes(), 119060324);
        assert_eq!(stats.captures(), 2812008);
        assert_eq!(stats.en_passants(), 5248);
        assert_eq!(stats.castles(), 0);
        assert_eq!(stats.promotions(), 0);
    }

    #[test]
    fn perft_startpos_7() {
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let mut cache = GenericTable::size(67108864);

        let mut board = Board::default(hasher);
        let stats = perft_stats::<true>(&mut board, &mut cache, 7);

        assert_eq!(stats.nodes(), 3195901860);
        assert_eq!(stats.captures(), 108329926);
        assert_eq!(stats.en_passants(), 319617);
        assert_eq!(stats.castles(), 883453);
        assert_eq!(stats.promotions(), 0);
    }

    #[test]
    fn perft_testsuit() {
        let mut file = File::open("test_data/perftsuite.epd").unwrap();

        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let mut cache = GenericTable::size(67108864);

        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let lines = contents.lines();
        for line in lines {
            let mut parts = line.split(" ;");
            let fen = parts.next().unwrap();
            if fen.starts_with("//") {
                continue;
            }

            println!("Starting perft of {}", fen);
            for depth in parts {
                let mut parts = depth.split(" ");
                let depth = parts.next().unwrap();
                let depth = depth.chars().nth(1).unwrap().to_digit(10).unwrap() as u8;
                let nodes = parts.next().unwrap().parse::<u64>().unwrap();

                println!(
                    " - Computing the amount of nodes for the depth of {}",
                    depth
                );

                let mut board = Board::from_str(fen, hasher.clone()).unwrap();
                let result = perft_normal::<true>(&mut board, &mut cache, depth);
                assert_eq!(result, nodes, "The computed amount of nodes {} for {} with the depth of {} doesn't match with the given node amount of {}", result, fen, depth, nodes);
            }
        }
    }
}
