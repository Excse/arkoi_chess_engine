use std::isize;

use crate::{
    board::{color::Color, piece::Piece, Board},
    move_generator::{mov::Move, MoveGenerator},
};

// https://www.chessprogramming.org/Simplified_Evaluation_Function
#[rustfmt::skip]
pub const PAWN_TABLE: [isize; Board::SIZE] = [
     0,  0,  0,  0,  0,  0,  0,  0,
    50, 50, 50, 50, 50, 50, 50, 50,
    10, 10, 20, 30, 30, 20, 10, 10,
     5,  5, 10, 25, 25, 10,  5,  5,
     0,  0,  0, 20, 20,  0,  0,  0,
     5, -5,-10,  0,  0,-10, -5,  5,
     5, 10, 10,-20,-20, 10, 10,  5,
     0,  0,  0,  0,  0,  0,  0,  0
];

#[rustfmt::skip]
pub const KNIGHT_TABLE: [isize; Board::SIZE] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,
];

#[rustfmt::skip]
pub const BISHOP_TABLE: [isize; Board::SIZE] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];

#[rustfmt::skip]
pub const ROOK_TABLE: [isize; Board::SIZE] = [
     0,  0,  0,  0,  0,  0,  0,  0,
     5, 10, 10, 10, 10, 10, 10,  5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
    -5,  0,  0,  0,  0,  0,  0, -5,
     0,  0,  0,  5,  5,  0,  0,  0
];

#[rustfmt::skip]
pub const QUEEN_TABLE: [isize; Board::SIZE] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5,  5,  5,  5,  0,-10,
     -5,  0,  5,  5,  5,  5,  0, -5,
      0,  0,  5,  5,  5,  5,  0, -5,
    -10,  5,  5,  5,  5,  5,  0,-10,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20
];

#[rustfmt::skip]
pub const KING_TABLE: [isize; Board::SIZE] = [
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -20,-30,-30,-40,-40,-30,-30,-20,
    -10,-20,-20,-20,-20,-20,-20,-10,
     20, 20,  0,  0,  0,  0, 20, 20,
     20, 30, 10,  0,  0, 10, 30, 20
];

pub fn piece_square(board: &Board, maximize: Color) -> isize {
    let mut eval = 0;

    let pawns = board.get_squares_by_piece(maximize, Piece::Pawn);
    for square in pawns {
        let index = if maximize == Color::Black {
            63 - square.index
        } else {
            square.index
        };

        eval += PAWN_TABLE[index]
    }

    let knights = board.get_squares_by_piece(maximize, Piece::Knight);
    for square in knights {
        let index = if maximize == Color::Black {
            63 - square.index
        } else {
            square.index
        };

        eval += KNIGHT_TABLE[index];
    }

    let bishops = board.get_squares_by_piece(maximize, Piece::Bishop);
    for square in bishops {
        let index = if maximize == Color::Black {
            63 - square.index
        } else {
            square.index
        };

        eval += BISHOP_TABLE[index];
    }

    let rooks = board.get_squares_by_piece(maximize, Piece::Rook);
    for square in rooks {
        let index = if maximize == Color::Black {
            63 - square.index
        } else {
            square.index
        };

        eval += ROOK_TABLE[index];
    }

    let queens = board.get_squares_by_piece(maximize, Piece::Queen);
    for square in queens {
        let index = if maximize == Color::Black {
            63 - square.index
        } else {
            square.index
        };

        eval += QUEEN_TABLE[index];
    }

    let kings = board.get_squares_by_piece(maximize, Piece::King);
    for square in kings {
        let index = if maximize == Color::Black {
            63 - square.index
        } else {
            square.index
        };

        eval += KING_TABLE[index];
    }

    eval
}

pub fn board_value(board: &Board, maximize: Color) -> isize {
    let mut eval = 0;

    let own = board.get_piece_count(maximize, Piece::Pawn) as isize;
    let other = board.get_piece_count(!maximize, Piece::Pawn) as isize;
    eval += (own - other) * 100;

    let own = board.get_piece_count(maximize, Piece::Knight) as isize;
    let other = board.get_piece_count(!maximize, Piece::Knight) as isize;
    eval += (own - other) * 320;

    let own = board.get_piece_count(maximize, Piece::Bishop) as isize;
    let other = board.get_piece_count(!maximize, Piece::Bishop) as isize;
    eval += (own - other) * 330;

    let own = board.get_piece_count(maximize, Piece::Rook) as isize;
    let other = board.get_piece_count(!maximize, Piece::Rook) as isize;
    eval += (own - other) * 500;

    let own = board.get_piece_count(maximize, Piece::Queen) as isize;
    let other = board.get_piece_count(!maximize, Piece::Queen) as isize;
    eval += (own - other) * 900;

    let own = board.get_piece_count(maximize, Piece::King) as isize;
    let other = board.get_piece_count(!maximize, Piece::King) as isize;
    eval += (own - other) * 20000;

    eval
}

pub fn evaluate(board: &Board, maximize: Color) -> isize {
    let mut eval = 0;

    eval += board_value(board, maximize);
    eval += piece_square(board, maximize);

    eval
}

pub fn minimax(
    board: &Board,
    move_generator: &MoveGenerator,
    start_depth: usize,
    depth: usize,
    mut alpha: isize,
    mut beta: isize,
    maximize: Color,
) -> (isize, Option<Move>) {
    if depth == 0 {
        return (evaluate(board, maximize), None);
    }

    let moves = move_generator.get_legal_moves(board).unwrap();
    if moves.is_empty() {
        let board_eval = evaluate(board, maximize);
        let depth = start_depth - depth.min(start_depth);

        let mut eval;
        if board.active == maximize {
            eval = std::isize::MIN;
            eval += depth as isize * 1_000_000;
            eval += board_eval;
        } else {
            eval = std::isize::MAX;
            eval -= depth as isize * 1_000_000;
            eval += board_eval;
        }

        return (eval, None);
    }

    if board.active == maximize {
        let mut max_eval = std::isize::MIN;
        let mut max_move = None;

        for mov in moves {
            let mut board = board.clone();
            board.make(&mov).unwrap();

            let (eval, _) = minimax(
                &board,
                move_generator,
                start_depth,
                depth - 1,
                alpha,
                beta,
                maximize,
            );
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
        let mut min_eval = std::isize::MAX;
        let mut min_move = None;

        for mov in moves {
            let mut board = board.clone();
            board.make(&mov).unwrap();

            let (eval, _) = minimax(
                &board,
                move_generator,
                start_depth,
                depth - 1,
                alpha,
                beta,
                maximize,
            );
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
