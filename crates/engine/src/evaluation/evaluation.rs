use base::board::{color::Color, piece::Piece, Board};

pub fn evaluate(board: &Board, active: Color) -> i32 {
    let mut eval = 0;

    eval += pesto_evaluation(board, active);
    eval += get_bishop_pair_difference(board, active);
    eval += get_rook_pair_difference(board, active);

    eval
}

pub(crate) fn pesto_evaluation(board: &Board, active: Color) -> i32 {
    let midgame_score = board.midgame(active) - board.midgame(active.other());
    let endgame_score = board.endgame(active) - board.endgame(active.other());

    let mut midgame_phase = board.gamephase();
    if midgame_phase > 24 {
        midgame_phase = 24;
    }
    let endgame_phase = 24 - midgame_phase;

    let mut eval = midgame_score * midgame_phase;
    eval += endgame_score * endgame_phase;
    eval /= 24;

    eval
}

pub(crate) fn get_bishop_pair_difference(board: &Board, active: Color) -> i32 {
    let mut eval = 0;

    eval += get_bishop_pair_eval(board, active);
    eval -= get_bishop_pair_eval(board, active.other());

    eval
}

pub(crate) fn get_bishop_pair_eval(board: &Board, color: Color) -> i32 {
    let mut eval = 0;

    let active_bishops = board.get_piece_count(color, Piece::Bishop);
    if active_bishops >= 2 {
        eval += 50;
    }

    eval
}

pub(crate) fn get_rook_pair_difference(board: &Board, active: Color) -> i32 {
    let mut eval = 0;

    eval += get_rook_pair_eval(board, active);
    eval -= get_rook_pair_eval(board, active.other());

    eval
}

pub(crate) fn get_rook_pair_eval(board: &Board, color: Color) -> i32 {
    let mut rank_counter = vec![0; 8];
    let mut file_counter = vec![0; 8];
    let mut eval = 0;

    let active_rooks = board.get_squares_by_piece(color, Piece::Rook);
    for rook in active_rooks {
        let rank = rook.rank() as usize;
        let file = rook.file() as usize;

        rank_counter[rank] += 1;
        file_counter[file] += 1;
    }

    let mut pairs = 0;
    for index in 0..8 {
        if rank_counter[index] >= 2 {
            pairs += 1;
        }

        if file_counter[index] >= 2 {
            pairs += 1;
        }
    }

    eval -= pairs * 50;

    eval
}
