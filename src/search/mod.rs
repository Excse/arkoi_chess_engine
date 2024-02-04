use crate::{
    board::{color::Color, piece::Piece, Board},
    move_generator::{mov::Move, MoveGenerator},
};

pub fn evaluate(board: &Board, maximize: Color) -> f64 {
    let mut eval = 0.0;

    let own = board.get_piece_count(maximize, Piece::Pawn) as f64;
    let other = board.get_piece_count(!maximize, Piece::Pawn) as f64;
    eval += (own - other) * 1.0;

    let own = board.get_piece_count(maximize, Piece::Knight) as f64;
    let other = board.get_piece_count(!maximize, Piece::Knight) as f64;
    eval += (own - other) * 3.0;

    let own = board.get_piece_count(maximize, Piece::Bishop) as f64;
    let other = board.get_piece_count(!maximize, Piece::Bishop) as f64;
    eval += (own - other) * 3.0;

    let own = board.get_piece_count(maximize, Piece::Rook) as f64;
    let other = board.get_piece_count(!maximize, Piece::Rook) as f64;
    eval += (own - other) * 5.0;

    let own = board.get_piece_count(maximize, Piece::Queen) as f64;
    let other = board.get_piece_count(!maximize, Piece::Queen) as f64;
    eval += (own - other) * 9.0;

    eval
}

pub fn minimax(
    board: &Board,
    move_generator: &MoveGenerator,
    depth: usize,
    mut alpha: f64,
    mut beta: f64,
    maximize: Color,
) -> (f64, Option<Move>) {
    if depth == 0 {
        return (evaluate(board, maximize), None);
    }

    let moves = move_generator.get_legal_moves(board).unwrap();
    if moves.is_empty() {
        return (evaluate(board, maximize), None);
    }

    if board.active == maximize {
        let mut max_eval = std::f64::NEG_INFINITY;
        let mut max_move = None;

        for mov in moves {
            let mut board = board.clone();
            board.make(&mov).unwrap();

            let (eval, _) = minimax(&board, move_generator, depth - 1, alpha, beta, maximize);
            if eval > max_eval {
                max_eval = eval;
                max_move = Some(mov);
            }

            alpha = alpha.max(max_eval);
            if beta <= alpha {
                break;
            }
        }

        (max_eval, max_move)
    } else {
        let mut min_eval = std::f64::INFINITY;
        let mut min_move = None;

        for mov in moves {
            let mut board = board.clone();
            board.make(&mov).unwrap();

            let (eval, _) = minimax(&board, move_generator, depth - 1, alpha, beta, maximize);
            if eval < min_eval {
                min_eval = eval;
                min_move = Some(mov);
            }

            beta = beta.min(min_eval);
            if beta <= alpha {
                break;
            }
        }

        (min_eval, min_move)
    }
}
