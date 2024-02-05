use std::isize;

use crate::{
    bitboard::square::Square,
    board::{color::Color, Board},
    move_generator::{mov::Move, MoveGenerator},
};

pub fn pesto_evaluation(board: &Board, maximize: Color) -> isize {
    let mut midgame = [0; Color::COUNT];
    let mut endgame = [0; Color::COUNT];
    let mut gamephase = 0;

    for square_index in 0..Board::SIZE {
        let square = Square::index(square_index);
        let colored_piece = match board.get_piece_type(square) {
            Some(colored_piece) => colored_piece,
            None => continue,
        };

        let mut value = colored_piece.get_midgame_square_value(square);
        value += colored_piece.piece.get_midgame_value();
        midgame[colored_piece.color.index()] += value;

        let mut value = colored_piece.get_endgame_square_value(square);
        value += colored_piece.piece.get_endgame_value();
        endgame[colored_piece.color.index()] += value;

        gamephase += colored_piece.piece.get_gamephase_value();
    }

    let midgame_score = midgame[maximize.index()] - midgame[(!maximize).index()];
    let endgame_score = endgame[maximize.index()] - endgame[(!maximize).index()];

    let mut midgame_phase = gamephase;
    if midgame_phase > 24 {
        midgame_phase = 24;
    }
    let endgame_phase = 24 - midgame_phase;

    let mut eval = midgame_score * midgame_phase;
    eval += endgame_score * endgame_phase;
    eval /= 24;

    eval
}

pub fn evaluate(board: &Board, maximize: Color) -> isize {
    let mut eval = 0;

    eval += pesto_evaluation(board, maximize);

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