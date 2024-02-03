#[cfg(test)]
mod perft {
    use std::{fs::File, io::Read, ops::AddAssign};

    use rand::{rngs::StdRng, SeedableRng};

    use crate::{
        board::{zobrist::ZobristHasher, Board},
        move_generator::{error::MoveGeneratorError, mov::MoveKind, MoveGenerator},
    };

    #[test]
    fn perft_startpos_0() {
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let board = Board::default(&hasher);
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 0).unwrap();
        assert_eq!(result.nodes, 1);
        assert_eq!(result.captures, 0);
        assert_eq!(result.en_passants, 0);
        assert_eq!(result.castles, 0);
        assert_eq!(result.promotions, 0);
        assert_eq!(result.checks, 0);
        assert_eq!(result.double_checks, 0);
    }

    #[test]
    fn perft_startpos_1() {
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let board = Board::default(&hasher);
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 1).unwrap();
        assert_eq!(result.nodes, 20);
        assert_eq!(result.captures, 0);
        assert_eq!(result.en_passants, 0);
        assert_eq!(result.castles, 0);
        assert_eq!(result.promotions, 0);
        assert_eq!(result.checks, 0);
        assert_eq!(result.double_checks, 0);
    }

    #[test]
    fn perft_startpos_2() {
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let board = Board::default(&hasher);
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 2).unwrap();
        assert_eq!(result.nodes, 400);
        assert_eq!(result.captures, 0);
        assert_eq!(result.en_passants, 0);
        assert_eq!(result.castles, 0);
        assert_eq!(result.promotions, 0);
        assert_eq!(result.checks, 0);
        assert_eq!(result.double_checks, 0);
    }

    #[test]
    fn perft_startpos_3() {
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let board = Board::default(&hasher);
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 3).unwrap();
        assert_eq!(result.nodes, 8902);
        assert_eq!(result.captures, 34);
        assert_eq!(result.en_passants, 0);
        assert_eq!(result.castles, 0);
        assert_eq!(result.promotions, 0);
        assert_eq!(result.checks, 12);
        assert_eq!(result.double_checks, 0);
    }

    #[test]
    fn perft_startpos_4() {
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let board = Board::default(&hasher);
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 4).unwrap();
        assert_eq!(result.nodes, 197281);
        assert_eq!(result.captures, 1576);
        assert_eq!(result.en_passants, 0);
        assert_eq!(result.castles, 0);
        assert_eq!(result.promotions, 0);
        assert_eq!(result.checks, 469);
        assert_eq!(result.double_checks, 0);
    }

    #[test]
    fn perft_startpos_5() {
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let board = Board::default(&hasher);
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 5).unwrap();
        assert_eq!(result.nodes, 4865609);
        assert_eq!(result.captures, 82719);
        assert_eq!(result.en_passants, 258);
        assert_eq!(result.castles, 0);
        assert_eq!(result.promotions, 0);
        assert_eq!(result.checks, 27351);
        assert_eq!(result.double_checks, 0);
    }

    #[test]
    fn perft_startpos_6() {
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let board = Board::default(&hasher);
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 6).unwrap();
        assert_eq!(result.nodes, 119060324);
        assert_eq!(result.captures, 2812008);
        assert_eq!(result.en_passants, 5248);
        assert_eq!(result.castles, 0);
        assert_eq!(result.promotions, 0);
        assert_eq!(result.checks, 809099);
        assert_eq!(result.double_checks, 46);
    }

    #[test]
    fn perft_startpos_7() {
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let board = Board::default(&hasher);
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 7).unwrap();
        assert_eq!(result.nodes, 3195901860);
        assert_eq!(result.captures, 108329926);
        assert_eq!(result.en_passants, 319617);
        assert_eq!(result.castles, 883453);
        assert_eq!(result.promotions, 0);
        assert_eq!(result.checks, 33103848);
        assert_eq!(result.double_checks, 1628);
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
                let move_generator = MoveGenerator::default();
                let result = perft(&board, &move_generator, depth).unwrap();
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
        checks: usize,
        double_checks: usize,
    }

    impl PerftResult {
        pub fn empty() -> Self {
            Self {
                nodes: 0,
                captures: 0,
                en_passants: 0,
                castles: 0,
                promotions: 0,
                checks: 0,
                double_checks: 0,
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
                checks: 0,
                double_checks: 0,
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
            self.checks += rhs.checks;
            self.double_checks += rhs.double_checks;
        }
    }

    fn perft(
        board: &Board,
        move_generator: &MoveGenerator,
        depth: u32,
    ) -> Result<PerftResult, MoveGeneratorError> {
        if depth == 0 {
            return Ok(PerftResult::default());
        }

        let mut result = PerftResult::empty();

        let moves = move_generator.get_legal_moves(board)?;
        for mov in moves {
            let mut board = board.clone();
            board.make(&mov)?;

            if depth == 1 {
                let king = board.get_king_square(board.active).unwrap();
                let checkers = move_generator.get_checkers(&board, king).len();
                if checkers != 0 {
                    result.checks += 1;
                }

                if checkers == 2 {
                    result.double_checks += 1;
                }

                // TODO: Add discovery checks and checkmate

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

            let next_perft = perft(&board, move_generator, depth - 1)?;
            result += next_perft;
        }

        Ok(result)
    }
}
