#[cfg(test)]
mod polyglot_hash {
    use crate::{board::Board, polyglot::hasher::PolyglotHasher, zobrist::ZobristHasher};

    #[test]
    fn starting_position() {
        let mut rng = rand::thread_rng();
        let hasher = ZobristHasher::random(&mut rng);

        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let board = Board::from_str(fen, hasher).unwrap();

        assert_eq!(PolyglotHasher::hash(&board), 0x463b96181691fc9c);
    }

    #[test]
    fn e2e4() {
        let mut rng = rand::thread_rng();
        let hasher = ZobristHasher::random(&mut rng);

        let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let board = Board::from_str(fen, hasher).unwrap();

        assert_eq!(PolyglotHasher::hash(&board), 0x823c9b50fd114196);
    }

    #[test]
    fn e2e4_d7d5() {
        let mut rng = rand::thread_rng();
        let hasher = ZobristHasher::random(&mut rng);

        let fen = "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2";
        let board = Board::from_str(fen, hasher).unwrap();

        assert_eq!(PolyglotHasher::hash(&board), 0x0756b94461c50fb0);
    }

    #[test]
    fn e2e4_d7d5_e4e5() {
        let mut rng = rand::thread_rng();
        let hasher = ZobristHasher::random(&mut rng);

        let fen = "rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 2";
        let board = Board::from_str(fen, hasher).unwrap();

        assert_eq!(PolyglotHasher::hash(&board), 0x662fafb965db29d4);
    }

    #[test]
    fn e2e4_d7d5_e4e5_f7f5() {
        let mut rng = rand::thread_rng();
        let hasher = ZobristHasher::random(&mut rng);

        let fen = "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3";
        let board = Board::from_str(fen, hasher).unwrap();

        assert_eq!(PolyglotHasher::hash(&board), 0x22a48b5a8e47ff78);
    }

    #[test]
    fn e2e4_d7d5_e4e5_f7f5_e1e2() {
        let mut rng = rand::thread_rng();
        let hasher = ZobristHasher::random(&mut rng);

        let fen = "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPPKPPP/RNBQ1BNR b kq - 0 3";
        let board = Board::from_str(fen, hasher).unwrap();

        assert_eq!(PolyglotHasher::hash(&board), 0x652a607ca3f242c1);
    }

    #[test]
    fn e2e4_d7d5_e4e5_f7f5_e1e2_e8f7() {
        let mut rng = rand::thread_rng();
        let hasher = ZobristHasher::random(&mut rng);

        let fen = "rnbq1bnr/ppp1pkpp/8/3pPp2/8/8/PPPPKPPP/RNBQ1BNR w - - 0 4";
        let board = Board::from_str(fen, hasher).unwrap();

        assert_eq!(PolyglotHasher::hash(&board), 0x00fdd303c946bdd9);
    }

    #[test]
    fn a2a4_b7b5_h2h4_b5b4_c2c4() {
        let mut rng = rand::thread_rng();
        let hasher = ZobristHasher::random(&mut rng);

        let fen = "rnbqkbnr/p1pppppp/8/8/PpP4P/8/1P1PPPP1/RNBQKBNR b KQkq c3 0 3";
        let board = Board::from_str(fen, hasher).unwrap();

        assert_eq!(PolyglotHasher::hash(&board), 0x3c8123ea7b067637);
    }

    #[test]
    fn a2a4_b7b5_h2h4_b5b4_c2c4_b4c3_a1a3() {
        let mut rng = rand::thread_rng();
        let hasher = ZobristHasher::random(&mut rng);

        let fen = "rnbqkbnr/p1pppppp/8/8/P6P/R1p5/1P1PPPP1/1NBQKBNR b Kkq - 0 4";
        let board = Board::from_str(fen, hasher).unwrap();

        assert_eq!(PolyglotHasher::hash(&board), 0x5c3f9b829b279560);
    }
}

#[cfg(test)]
mod polyglot_parser {
    use std::{fs::File, io::Read};

    use crate::{polyglot::parser::PolyglotBook, zobrist::ZobristHasher, board::Board};

    #[test]
    fn can_parse_perfect() {
        let mut file = File::open("test_data/Perfect2023.bin").unwrap();

        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();

        let book = PolyglotBook::parse(&data).unwrap();

        let mut rng = rand::thread_rng();
        let hasher = ZobristHasher::random(&mut rng);

        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let board = Board::from_str(fen, hasher).unwrap();

        let moves = book.get_entries(&board).unwrap();
        assert_eq!(moves.len(), 4);
    }
}
