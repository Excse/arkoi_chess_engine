#[cfg(test)]
mod perft {
    use std::{fs::File, io::Read, ops::AddAssign};

    use crate::{
        board::{zobrist::ZobristHasher, Board},
        move_generator::{error::MoveGeneratorError, mov::MoveKind, MoveGenerator},
    };

    #[test]
    fn perft_startpos_0() {
        let hasher = ZobristHasher::new();
        let board = Board::default(&hasher);
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 0).unwrap();
        assert_eq!(result.nodes, 1);
        assert_eq!(result.castles, 0);
        assert_eq!(result.en_passants, 0);
    }

    #[test]
    fn perft_startpos_1() {
        let hasher = ZobristHasher::new();
        let board = Board::default(&hasher);
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 1).unwrap();
        assert_eq!(result.nodes, 20);
        assert_eq!(result.castles, 0);
        assert_eq!(result.en_passants, 0);
    }

    #[test]
    fn perft_startpos_2() {
        let hasher = ZobristHasher::new();
        let board = Board::default(&hasher);
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 2).unwrap();
        assert_eq!(result.nodes, 400);
        assert_eq!(result.castles, 0);
        assert_eq!(result.en_passants, 0);
    }

    #[test]
    fn perft_startpos_3() {
        let hasher = ZobristHasher::new();
        let board = Board::default(&hasher);
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 3).unwrap();
        assert_eq!(result.nodes, 8902);
        assert_eq!(result.castles, 0);
        assert_eq!(result.en_passants, 0);
    }

    #[test]
    fn perft_startpos_4() {
        let hasher = ZobristHasher::new();
        let board = Board::default(&hasher);
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 4).unwrap();
        assert_eq!(result.nodes, 197281);
        assert_eq!(result.castles, 0);
        assert_eq!(result.en_passants, 0);
    }

    #[test]
    fn perft_startpos_5() {
        let hasher = ZobristHasher::new();
        let board = Board::default(&hasher);
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 5).unwrap();
        assert_eq!(result.nodes, 4865609);
        assert_eq!(result.castles, 0);
        assert_eq!(result.en_passants, 258);
    }

    #[test]
    fn perft_testsuit() {
        let mut file = File::open("perftsuite.epd").unwrap();

        let hasher = ZobristHasher::new();
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
        castles: usize,
        en_passants: usize,
    }

    impl PerftResult {
        pub fn empty() -> Self {
            Self {
                nodes: 0,
                captures: 0,
                castles: 0,
                en_passants: 0,
            }
        }
    }

    impl Default for PerftResult {
        fn default() -> Self {
            Self {
                nodes: 1,
                captures: 0,
                castles: 0,
                en_passants: 0,
            }
        }
    }

    impl AddAssign for PerftResult {
        fn add_assign(&mut self, rhs: Self) {
            self.nodes += rhs.nodes;
            self.captures += rhs.captures;
            self.castles += rhs.castles;
            self.en_passants += rhs.en_passants;
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
            match mov.kind {
                MoveKind::Castle(_) => result.castles += 1,
                MoveKind::EnPassant(_) => result.en_passants += 1,
                _ => {}
            }

            let mut board = board.clone();
            board.play(board.active, &mov)?;
            board.swap_active();

            let next_perft = perft(&board, move_generator, depth - 1)?;
            result += next_perft;
        }

        Ok(result)
    }
}
