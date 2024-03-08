use base::board::{color::Color, Board};

pub fn evaluate(board: &Board, active: Color) -> i32 {
    let mut eval = 0;

    eval += pesto_evaluation(board, active);
    eval += get_bishop_pair_difference(board, active);

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

    if board.has_bishop_pair(color) {
        eval += 50;
    }

    eval
}
