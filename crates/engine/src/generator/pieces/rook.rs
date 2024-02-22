use base::{
    bitboard::Bitboard,
    board::{piece::Piece, Board},
    r#move::Move,
    square::Square,
};

use crate::generator::{CheckType, MoveGenerator, PieceGenerator};

pub(crate) struct RookGenerator;

impl PieceGenerator for RookGenerator {
    #[inline(always)]
    fn pseudo_legals(
        _board: &Board,
        source: Square,
        allowed: Bitboard,
        blockers: Bitboard,
    ) -> Bitboard {
        let mut pseudo_legals = Bitboard::default();

        pseudo_legals |= source.get_rook_attacks(blockers);

        pseudo_legals &= allowed;
        pseudo_legals
    }

    #[inline(always)]
    fn legals<T>(generator: &mut MoveGenerator, board: &Board)
    where
        T: CheckType,
    {
        let rooks = board.get_piece_board(board.active(), Piece::Rook);
        let own_occupied = board.get_occupied(board.active());
        let all_occupied = board.get_all_occupied();
        let pinned = board.pinned();
        let unpinned = !pinned;

        // All the moves that are allowed based on the check condition.
        let check_mask = if T::IN_CHECK {
            let king_square = board.get_king_square(board.active());

            // There can only be one checker, otherwise we would only calculate
            // king moves.
            let checker = Square::from(board.checkers());

            // Get all the bits between the checker and the king, the checker is
            // inclusive and the king is exclusive.
            let check_mask = checker.get_between(king_square) ^ checker;

            check_mask
        } else {
            // We want to allow all moves if we are not in check.
            Bitboard::ALL_BITS
        };

        // All moves are valid where no own piece is on the destination and
        // the checkmask is set.
        let allowed = !own_occupied & check_mask;

        // At first calculate every rook move that is not pinned.
        for source in rooks & unpinned {
            // Now we can generate all pseudo legal moves for the rook and
            // be sure that they are legal.
            let moves = Self::pseudo_legals(board, source, allowed, all_occupied);

            // Extract all the squares and add the moves to the move list.
            let moves = moves.get_squares();
            for target in moves {
                // If there is a piece on the target square, we capture it.
                let captured_piece = match board.get_piece_type(target) {
                    Some(colored_piece) => colored_piece.piece,
                    None => Piece::None,
                };

                // Create a potential capture move. At the end it doesn't matter if
                // the captured square is set or not.
                let mov = Move::capture(Piece::Rook, source, target, captured_piece);
                generator.push(mov);
            }
        }

        // It is not possible to move pinned pieces when in check
        if !T::IN_CHECK {
            let king_square = board.get_king_square(board.active());

            // If not in check we calculate every move for pinned pieces
            for source in rooks & pinned {
                // The line of the rook to the king.
                let line = king_square.get_line(source);
                // We just can move on the line. This will allow us to generate
                // every move between the pinner and the king, but also the capture
                // move of the pinner without interfering with the other pinned lines.
                let allowed = allowed & line;

                // Now we can generate all pseudo legal moves for the rook and
                // be sure that they are legal.
                let moves = Self::pseudo_legals(board, source, allowed, all_occupied);

                // Extract all the squares and add the moves to the move list.
                let moves = moves.get_squares();
                for target in moves {
                    // If there is a piece on the target square, we capture it.
                    let captured_piece = match board.get_piece_type(target) {
                        Some(colored_piece) => colored_piece.piece,
                        None => Piece::None,
                    };

                    // Create a potential capture move. At the end it doesn't matter if
                    // the captured square is set or not.
                    let mov = Move::capture(Piece::Rook, source, target, captured_piece);
                    generator.push(mov);
                }
            }
        }
    }
}