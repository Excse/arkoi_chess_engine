#[cfg(test)]
mod perft {
    use std::ops::AddAssign;

    use crate::{
        board::Board,
        move_generator::{error::MoveGeneratorError, Move, MoveGenerator},
    };

    #[test]
    fn perft_startpos_0() {
        let board = Board::default();
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 0).unwrap();
        assert_eq!(result.nodes, 1);
        assert_eq!(result.captures, 0);
        assert_eq!(result.castles, 0);
    }

    #[test]
    fn perft_startpos_1() {
        let board = Board::default();
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 1).unwrap();
        assert_eq!(result.nodes, 20);
        assert_eq!(result.captures, 0);
        assert_eq!(result.castles, 0);
    }

    #[test]
    fn perft_startpos_2() {
        let board = Board::default();
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 2).unwrap();
        assert_eq!(result.nodes, 400);
        assert_eq!(result.captures, 0);
        assert_eq!(result.castles, 0);
    }

    #[test]
    fn perft_startpos_3() {
        let board = Board::default();
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 3).unwrap();
        assert_eq!(result.nodes, 8902);
        assert_eq!(result.captures, 34);
        assert_eq!(result.castles, 0);
    }

    #[test]
    fn perft_startpos_4() {
        let board = Board::default();
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 4).unwrap();
        assert_eq!(result.nodes, 197281);
        assert_eq!(result.captures, 1576);
        assert_eq!(result.castles, 0);
    }

    #[test]
    fn perft_startpos_5() {
        let board = Board::default();
        let move_generator = MoveGenerator::default();
        let result = perft(&board, &move_generator, 5).unwrap();
        assert_eq!(result.nodes, 4865609);
        assert_eq!(result.captures, 82719);
        assert_eq!(result.castles, 0);
    }

    #[derive(Default, Debug)]
    struct PerftResult {
        nodes: usize,
        captures: usize,
        castles: usize,
    }

    impl AddAssign for PerftResult {
        fn add_assign(&mut self, rhs: Self) {
            self.nodes += rhs.nodes;
            self.captures += rhs.captures;
            self.castles += rhs.castles;
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
            });
        }

        let mut result = PerftResult::default();

        let moves = move_generator.get_legal_moves(board)?;
        if depth == 1 {
            result.nodes = moves.len();
            return Ok(result);
        }

        for mov in moves {
            // TODO: Implement this
            // if mov.attack {
            //     result.captures += 1;
            // }

            // match mov {
            //     Move::OOO_KING_WHITE
            //     | Move::OO_KING_WHITE
            //     | Move::OO_KING_BLACK
            //     | Move::OOO_KING_BLACK => result.castles += 1,
            //     _ => {}
            // }

            let mut board = board.clone();
            board.play(board.active, &mov)?;
            board.swap_active();

            let next_perft = perft(&board, move_generator, depth - 1)?;
            result += next_perft;
        }

        Ok(result)
    }
}
