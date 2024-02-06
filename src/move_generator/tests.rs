#[cfg(test)]
mod perft {
    use std::{fs::File, io::Read, ops::AddAssign};

    use rand::{rngs::StdRng, SeedableRng};

    use crate::{
        board::{zobrist::ZobristHasher, Board},
        move_generator::{error::MoveGeneratorError, mov::MoveKind},
    };

    #[test]
    fn perft_startpos_0() {
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let board = Board::default(&hasher);
        let result = perft(&board, 0).unwrap();
        assert_eq!(result.nodes, 1);
        assert_eq!(result.captures, 0);
        assert_eq!(result.en_passants, 0);
        assert_eq!(result.castles, 0);
        assert_eq!(result.promotions, 0);
    }

    #[test]
    fn perft_startpos_1() {
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let board = Board::default(&hasher);
        let result = perft(&board, 1).unwrap();
        assert_eq!(result.nodes, 20);
        assert_eq!(result.captures, 0);
        assert_eq!(result.en_passants, 0);
        assert_eq!(result.castles, 0);
        assert_eq!(result.promotions, 0);
    }

    #[test]
    fn perft_startpos_2() {
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let board = Board::default(&hasher);
        let result = perft(&board, 2).unwrap();
        assert_eq!(result.nodes, 400);
        assert_eq!(result.captures, 0);
        assert_eq!(result.en_passants, 0);
        assert_eq!(result.castles, 0);
        assert_eq!(result.promotions, 0);
    }

    #[test]
    fn perft_startpos_3() {
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let board = Board::default(&hasher);
        let result = perft(&board, 3).unwrap();
        assert_eq!(result.nodes, 8902);
        assert_eq!(result.captures, 34);
        assert_eq!(result.en_passants, 0);
        assert_eq!(result.castles, 0);
        assert_eq!(result.promotions, 0);
    }

    #[test]
    fn perft_startpos_4() {
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let board = Board::default(&hasher);
        let result = perft(&board, 4).unwrap();
        assert_eq!(result.nodes, 197281);
        assert_eq!(result.captures, 1576);
        assert_eq!(result.en_passants, 0);
        assert_eq!(result.castles, 0);
        assert_eq!(result.promotions, 0);
    }

    #[test]
    fn perft_startpos_5() {
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let board = Board::default(&hasher);
        let result = perft(&board, 5).unwrap();
        assert_eq!(result.nodes, 4865609);
        assert_eq!(result.captures, 82719);
        assert_eq!(result.en_passants, 258);
        assert_eq!(result.castles, 0);
        assert_eq!(result.promotions, 0);
    }

    #[test]
    fn perft_startpos_6() {
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let board = Board::default(&hasher);
        let result = perft(&board, 6).unwrap();
        assert_eq!(result.nodes, 119060324);
        assert_eq!(result.captures, 2812008);
        assert_eq!(result.en_passants, 5248);
        assert_eq!(result.castles, 0);
        assert_eq!(result.promotions, 0);
    }

    #[test]
    fn perft_startpos_7() {
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let board = Board::default(&hasher);
        let result = perft(&board, 7).unwrap();
        assert_eq!(result.nodes, 3195901860);
        assert_eq!(result.captures, 108329926);
        assert_eq!(result.en_passants, 319617);
        assert_eq!(result.castles, 883453);
        assert_eq!(result.promotions, 0);
    }

    #[test]
    fn perft_testsuit() {
        let mut file = File::open("perftsuite.epd").unwrap();

        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
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
                let depth = depth.chars().nth(1).unwrap().to_digit(10).unwrap();
                let nodes = parts.next().unwrap().parse::<usize>().unwrap();

                println!(
                    " - Computing the amount of nodes for the depth of {}",
                    depth
                );

                let board = Board::from_str(fen, &hasher).unwrap();
                let result = perft(&board, depth).unwrap();
                assert_eq!(result.nodes, nodes, "The computed amount of nodes {} for {} with the depth of{} doesn't match with the given node amount of {}", result.nodes, fen, depth, nodes);
            }
        }

        println!("Opened file");
    }

    #[derive(Debug)]
    struct PerftResult {
        nodes: usize,
        captures: usize,
        en_passants: usize,
        castles: usize,
        promotions: usize,
    }

    impl PerftResult {
        pub fn empty() -> Self {
            Self {
                nodes: 0,
                captures: 0,
                en_passants: 0,
                castles: 0,
                promotions: 0,
            }
        }
    }

    impl Default for PerftResult {
        fn default() -> Self {
            Self {
                nodes: 1,
                captures: 0,
                en_passants: 0,
                castles: 0,
                promotions: 0,
            }
        }
    }

    impl AddAssign for PerftResult {
        fn add_assign(&mut self, rhs: Self) {
            self.nodes += rhs.nodes;
            self.captures += rhs.captures;
            self.en_passants += rhs.en_passants;
            self.castles += rhs.castles;
            self.promotions += rhs.promotions;
        }
    }

    fn perft(board: &Board, depth: u32) -> Result<PerftResult, MoveGeneratorError> {
        if depth == 0 {
            return Ok(PerftResult::default());
        }

        let mut result = PerftResult::empty();

        let move_state = board.get_legal_moves()?;
        for mov in move_state.moves {
            if depth == 1 {
                match mov.kind {
                    MoveKind::Castle(_) => result.castles += 1,
                    MoveKind::EnPassant(_) => {
                        result.en_passants += 1;
                        result.captures += 1;
                    }
                    MoveKind::Attack => {
                        result.captures += 1;
                    }
                    MoveKind::Promotion(ref mov) => {
                        result.promotions += 1;
                        if mov.is_attack {
                            result.captures += 1;
                        }
                    }
                    _ => {}
                }

                result.nodes += 1;
                continue;
            }

            let mut board = board.clone();
            board.make(&mov)?;

            let next_perft = perft(&board, depth - 1)?;
            result += next_perft;
        }

        Ok(result)
    }
}
