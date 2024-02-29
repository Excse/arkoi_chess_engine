use base::{
    bitboard::Bitboard,
    board::{piece::Piece, Board},
    r#move::Move,
    square::Square,
};

use crate::generator::{CheckType, MoveGenerator, MoveType, PieceGenerator};

pub(crate) struct KnightGenerator;

impl PieceGenerator for KnightGenerator {
    #[inline(always)]
    fn pseudo_legals(
        _board: &Board,
        source: Square,
        allowed: Bitboard,
        _blockers: Bitboard,
    ) -> Bitboard {
        let mut pseudo_legals = Bitboard::default();

        pseudo_legals |= source.get_knight_moves();

        pseudo_legals &= allowed;
        pseudo_legals
    }

    #[inline(always)]
    fn legals<C, M>(generator: &mut MoveGenerator<M>, board: &Board)
    where
        C: CheckType,
        M: MoveType,
    {
        let knights = board.get_piece_board(board.active(), Piece::Knight);
        let all_occupied = board.get_all_occupied();
        let unpinned = !board.pinned();

        // All the moves that are allowed based on the check condition.
        let check_mask = if C::IN_CHECK {
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
        let allowed = if M::QUIET && M::QUIET {
            let own_occupied = board.get_occupied(board.active());
            !own_occupied & check_mask
        } else if M::QUIET {
            !all_occupied & check_mask
        } else if M::CAPTURE {
            let them_occupied = board.get_occupied(board.other());
            them_occupied & check_mask
        } else {
            panic!("Invalid move type");
        };

        // The knight can never move on a pin ray because of it's movement.
        // Thus we can ignore every knight that is pinned.
        for source in knights & unpinned {
            // Now we can generate all pseudo legal moves for the knight and
            // be sure that they are legal.
            let moves = Self::pseudo_legals(board, source, allowed, all_occupied);

            // Extract all the squares and add the moves to the move list.
            let moves = moves.get_squares();
            for target in moves {
                if M::CAPTURE && M::QUIET {
                    let is_capture = board.get_tile(target).is_some();
                    if is_capture {
                        let mov = Move::capture(source, target);
                        generator.push(mov);
                        continue;
                    } else {
                        let mov = Move::quiet(source, target);
                        generator.push(mov);
                        continue;
                    }
                } else if M::CAPTURE {
                    let mov = Move::capture(source, target);
                    generator.push(mov);
                    continue;
                } else if M::QUIET {
                    let mov = Move::quiet(source, target);
                    generator.push(mov);
                    continue;
                }
            }
        }
    }
}
