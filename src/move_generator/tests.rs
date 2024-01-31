#[cfg(test)]
mod perft {
    use std::ops::AddAssign;

    use crate::{
        board::Board,
        move_generator::{error::MoveGeneratorError, mov::MoveKind, MoveGenerator},
    };

    #[test]
    fn perft_startpos_0() {
        let board = Board::default();
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 0).unwrap();
        assert_eq!(result.nodes, 1);
        assert_eq!(result.castles, 0);
        assert_eq!(result.en_passants, 0);
        assert_eq!(result.checks, 0);
    }

    #[test]
    fn perft_startpos_1() {
        let board = Board::default();
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 1).unwrap();
        assert_eq!(result.nodes, 20);
        assert_eq!(result.castles, 0);
        assert_eq!(result.en_passants, 0);
        assert_eq!(result.checks, 0);
    }

    #[test]
    fn perft_startpos_2() {
        let board = Board::default();
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 2).unwrap();
        assert_eq!(result.nodes, 400);
        assert_eq!(result.castles, 0);
        assert_eq!(result.en_passants, 0);
        assert_eq!(result.checks, 0);
    }

    #[test]
    fn perft_startpos_3() {
        let board = Board::default();
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 3).unwrap();
        assert_eq!(result.nodes, 8902);
        assert_eq!(result.castles, 0);
        assert_eq!(result.en_passants, 0);
        assert_eq!(result.checks, 12);
    }

    #[test]
    fn perft_startpos_4() {
        let board = Board::default();
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 4).unwrap();
        assert_eq!(result.nodes, 197281);
        assert_eq!(result.castles, 0);
        assert_eq!(result.en_passants, 0);
        assert_eq!(result.checks, 469);
    }

    #[test]
    fn perft_startpos_5() {
        let board = Board::default();
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 5).unwrap();
        assert_eq!(result.nodes, 4865609);
        assert_eq!(result.castles, 0);
        assert_eq!(result.en_passants, 258);
        assert_eq!(result.checks, 27351);
    }

    #[derive(Default, Debug)]
    struct PerftResult {
        nodes: usize,
        captures: usize,
        castles: usize,
        en_passants: usize,
        checks: usize,
    }

    impl AddAssign for PerftResult {
        fn add_assign(&mut self, rhs: Self) {
            self.nodes += rhs.nodes;
            self.captures += rhs.captures;
            self.castles += rhs.castles;
            self.en_passants += rhs.en_passants;
            self.checks += rhs.checks;
        }
    }

    fn perft(
        board: &Board,
        move_generator: &MoveGenerator,
        depth: u8,
    ) -> Result<PerftResult, MoveGeneratorError> {
        if depth == 0 {
            return Ok(PerftResult {
                nodes: 1,
                captures: 0,
                castles: 0,
                en_passants: 0,
                checks: 0,
            });
        }

        let mut result = PerftResult::default();

        let moves = move_generator.get_legal_moves(board)?;
        let king = board.get_king_square(board.active)?;
        // let checks = move_generator.get_checkers(board, king).len();
        // if result.checks == 1 {
        //     result.checks = checks;
        // } else {
        //     println!("{}", result.checks);
        // }

        // if depth == 1 {
        //     result.nodes = moves.len();
        //     return Ok(result);
        // }

        for mov in moves {
            // let is_attacking = mov.get_attacking_square().is_some();
            // TODO: Implement this
            // if is_attacking {
            //     result.captures += 1;
            // }

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
