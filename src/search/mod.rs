use crate::{
    bitboard::square::Square,
    board::{color::Color, Board},
    move_generator::mov::Move, hashtable::{HashTable, transposition::{TranspositionEntry, TranspositionFlag}},
};

fn pesto_evaluation(board: &Board) -> isize {
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

    let midgame_score = midgame[board.active.index()] - midgame[(!board.active).index()];
    let endgame_score = endgame[board.active.index()] - endgame[(!board.active).index()];

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

pub fn evaluate(board: &Board) -> isize {
    let mut eval = 0;

    eval += pesto_evaluation(board);

    eval
}

pub fn search(
    board: &Board,
    cache: &mut HashTable<TranspositionEntry>,
    depth: u8,
) -> (isize, Option<Move>) {
    let mut best_eval = std::isize::MIN;
    let mut best_move = None;

    let move_state = board.get_legal_moves().unwrap();
    for mov in move_state.moves {
        let mut board = board.clone();
        board.make(&mov).unwrap();

        let eval = -negamax(
            &board,
            cache,
            depth,
            depth - 1,
            std::isize::MIN,
            std::isize::MAX,
            false,
        );
        if eval > best_eval {
            best_eval = eval;
            best_move = Some(mov);
        }
    }

    (best_eval, best_move)
}

fn negamax(
    board: &Board,
    cache: &mut HashTable<TranspositionEntry>,
    start_depth: u8,
    mut depth: u8,
    mut alpha: isize,
    mut beta: isize,
    mut extended: bool,
) -> isize {
    if board.halfmoves >= 50 {
        return 0;
    }

    let move_state = board.get_legal_moves().unwrap();
    if move_state.is_stalemate {
        return 0;
    } else if move_state.is_checkmate {
        let depth = start_depth - depth.min(start_depth);

        let mut eval = std::isize::MIN;
        eval += depth as isize * 1_000_000;

        return eval;
    } else if depth == 0 {
        if move_state.is_check && !extended {
            depth += 1;
            extended = true;
        } else {
            return evaluate(board);
        }
    }

    let original_alpha = alpha;
    if let Some(entry) = cache.probe(board.hash) {
        if entry.depth >= depth {
            match entry.flag {
                TranspositionFlag::Exact => return entry.eval,
                TranspositionFlag::LowerBound => alpha = alpha.max(entry.eval),
                TranspositionFlag::UpperBound => beta = beta.min(entry.eval),
            }

            if alpha >= beta {
                return entry.eval;
            }
        }
    }

    let mut eval = std::isize::MIN;
    for mov in move_state.moves {
        let mut board = board.clone();
        board.make(&mov).unwrap();

        let leaf_eval = -negamax(
            &board,
            cache,
            start_depth,
            depth - 1,
            -beta,
            -alpha,
            extended,
        );
        eval = eval.max(leaf_eval);

        alpha = alpha.max(eval);
        if alpha >= beta {
            break;
        }
    }

    let flag = if eval <= original_alpha {
        TranspositionFlag::UpperBound
    } else if eval >= beta {
        TranspositionFlag::LowerBound
    } else {
        TranspositionFlag::Exact
    };

    let entry = TranspositionEntry::new(board.hash, depth, flag, eval);
    cache.store(entry);

    eval
}
