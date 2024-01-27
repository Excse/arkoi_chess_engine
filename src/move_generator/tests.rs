#[cfg(test)]
mod perft {
    use std::ops::AddAssign;

    use crate::{
        board::Board,
        move_generator::{error::MoveGeneratorError, MoveGenerator},
    };

    #[test]
    fn perft_startpos_0() {
        let board = Board::default();
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 0).unwrap();
        assert_eq!(result.nodes, 1);
        assert_eq!(result.captures, 0);
        assert_eq!(result.checks, 0);
        assert_eq!(result.double_checks, 0);
    }

    #[test]
    fn perft_startpos_1() {
        let board = Board::default();
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 1).unwrap();
        assert_eq!(result.nodes, 20);
        assert_eq!(result.captures, 0);
        assert_eq!(result.checks, 0);
        assert_eq!(result.double_checks, 0);
    }

    #[test]
    fn perft_startpos_2() {
        let board = Board::default();
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 2).unwrap();
        assert_eq!(result.nodes, 400);
        assert_eq!(result.captures, 0);
        assert_eq!(result.checks, 0);
        assert_eq!(result.double_checks, 0);
    }

    #[test]
    fn perft_startpos_3() {
        let board = Board::default();
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 3).unwrap();
        assert_eq!(result.nodes, 8902);
        assert_eq!(result.captures, 34);
        assert_eq!(result.checks, 12);
        assert_eq!(result.double_checks, 0);
    }

    #[test]
    fn perft_startpos_4() {
        let board = Board::default();
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 4).unwrap();
        assert_eq!(result.nodes, 197281);
        assert_eq!(result.captures, 1576);
        assert_eq!(result.checks, 469);
        assert_eq!(result.double_checks, 0);
    }

    #[test]
    fn perft_startpos_5() {
        let board = Board::default();
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 5).unwrap();
        assert_eq!(result.nodes, 4865609);
        assert_eq!(result.captures, 82719);
        assert_eq!(result.checks, 27351);
        assert_eq!(result.double_checks, 0);
    }

    #[derive(Default, Debug)]
    struct PerftResult {
        nodes: usize,
        checks: usize,
        double_checks: usize,
        captures: usize,
    }

    impl AddAssign for PerftResult {
        fn add_assign(&mut self, rhs: Self) {
            self.nodes += rhs.nodes;
            self.checks += rhs.checks;
            self.double_checks += rhs.double_checks;
            self.captures += rhs.captures;
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
                checks: 0,
                double_checks: 0,
                captures: 0,
            });
        }

        let mut result = PerftResult::default();

        let moves = move_generator.get_legal_moves(board)?;

        let king = board.get_king_square(board.active)?;
        let checkers = move_generator.get_checkers(board, king);
        if checkers.len() == 1 {
            result.checks += 1;
        } else if checkers.len() == 2 {
            result.double_checks += 1;
        }

        for mov in moves {
            let mut board = board.clone();
            board.play(board.active, &mov)?;
            board.swap_active();

            if mov.attack {
                result.captures += 1;
            }

            let next_perft = perft(&board, move_generator, depth - 1)?;
            result += next_perft;
        }

        Ok(result)
    }
}
