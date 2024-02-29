use base::{
    bitboard::Bitboard,
    board::{board::EnPassant, color::Color, piece::Piece, Board},
    r#move::Move,
    square::Square,
};

use crate::generator::{CheckType, MoveGenerator, MoveType, PieceGenerator};

pub(crate) struct PawnGenerator;

impl PieceGenerator for PawnGenerator {
    #[inline(always)]
    fn pseudo_legals(
        board: &Board,
        source: Square,
        allowed: Bitboard,
        blockers: Bitboard,
    ) -> Bitboard {
        let mut pseudo_legals = Bitboard::default();

        let mut double_push_blockers = blockers ^ source;
        match board.other() {
            Color::Black => double_push_blockers <<= 8,
            Color::White => double_push_blockers >>= 8,
        }

        let mut pushes = source.get_pawn_pushes(board.active());
        pushes &= !blockers;
        pushes &= !double_push_blockers;

        pseudo_legals |= pushes;

        let attacks = source.get_pawn_attacks(board.active());
        pseudo_legals |= attacks & blockers;

        pseudo_legals &= allowed;
        pseudo_legals
    }

    #[inline(always)]
    fn legals<T, M>(generator: &mut MoveGenerator<M>, board: &Board)
    where
        T: CheckType,
        M: MoveType,
    {
        let pawns = board.get_piece_board(board.active(), Piece::Pawn);
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

        // At first calculate every pawn move that is not pinned.
        for source in pawns & unpinned {
            // Now we can generate all pseudo legal moves for the pawn and
            // be sure that they are legal.
            let moves = Self::pseudo_legals(board, source, allowed, all_occupied);

            // Extract all the squares and add the moves to the move list.
            let moves = moves.get_squares();
            for target in moves {
                Self::push_pawn_moves(generator, board, source, target);
            }
        }

        // It is not possible to move pinned pieces when in check
        if !T::IN_CHECK {
            let king_square = board.get_king_square(board.active());

            // If not in check we calculate every move for pinned pieces
            for source in pawns & pinned {
                // The line of the pawn to the king.
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
                    Self::push_pawn_moves(generator, board, source, target);
                }
            }
        }

        if M::CAPTURE {
            if let Some(en_passant) = board.en_passant() {
                let to_capture = en_passant.to_capture;
                let destination = en_passant.to_move;

                let rank = to_capture.rank_bb();
                let adjacent_files = to_capture.get_adjacent_files();

                let allowed = pawns & rank & adjacent_files;
                for source in allowed {
                    if !Self::is_legal_en_passant(board, source, en_passant) {
                        continue;
                    }

                    let mov = Move::en_passant(source, destination);
                    generator.push(mov);
                }
            }
        }
    }
}

impl PawnGenerator {
    fn is_legal_en_passant(board: &Board, source: Square, en_passant: &EnPassant) -> bool {
        let king_square = board.get_king_square(board.active());
        let all_occupied = board.get_all_occupied();

        let mut blockers = all_occupied;
        blockers ^= source;
        blockers ^= en_passant.to_capture;
        blockers ^= en_passant.to_move;

        let mut attackers = Bitboard::default();

        let queens = board.get_piece_board(board.other(), Piece::Queen);

        let bishops = board.get_piece_board(board.other(), Piece::Bishop);
        let bishop_attacks = king_square.get_bishop_attacks(blockers);
        attackers ^= bishop_attacks & (bishops | queens);

        let rooks = board.get_piece_board(board.other(), Piece::Rook);
        let rook_attacks = king_square.get_rook_attacks(blockers);
        attackers ^= rook_attacks & (rooks | queens);

        attackers.is_empty()
    }

    fn is_double_pawn(from: Square, to: Square) -> bool {
        let source_rank = from.rank();

        let is_double_pawn = source_rank == 1 || source_rank == 6;
        if is_double_pawn {
            let diff = (i8::from(from) - i8::from(to)).abs();
            return diff == 16;
        }

        false
    }

    fn is_promotion(to: Square) -> bool {
        let target_rank = to.rank();
        target_rank == 0 || target_rank == 7
    }

    fn push_pawn_moves<M: MoveType>(
        generator: &mut MoveGenerator<M>,
        board: &Board,
        source: Square,
        target: Square,
    ) {
        let is_promotion = Self::is_promotion(target);

        if M::CAPTURE && M::QUIET {
            // If there is a piece on the target square, we capture it.
            let is_capture = board.get_tile(target).is_some();

            if is_promotion {
                generator.push(Move::promotion(source, target, Piece::Queen, is_capture));
                generator.push(Move::promotion(source, target, Piece::Rook, is_capture));
                generator.push(Move::promotion(source, target, Piece::Bishop, is_capture));
                generator.push(Move::promotion(source, target, Piece::Knight, is_capture));
                return;
            }

            if is_capture {
                let mov = Move::capture(source, target);
                generator.push(mov);
                return;
            }

            let is_double_pawn = Self::is_double_pawn(source, target);
            if is_double_pawn {
                let mov = Move::double_pawn(source, target);
                generator.push(mov);
                return;
            } else {
                let mov = Move::quiet(source, target);
                generator.push(mov);
                return;
            }
        } else if M::CAPTURE {
            if is_promotion {
                generator.push(Move::promotion(source, target, Piece::Queen, true));
                generator.push(Move::promotion(source, target, Piece::Rook, true));
                generator.push(Move::promotion(source, target, Piece::Bishop, true));
                generator.push(Move::promotion(source, target, Piece::Knight, true));
                return;
            } else {
                let mov = Move::capture(source, target);
                generator.push(mov);
                return;
            }
        } else if M::QUIET {
            if is_promotion {
                generator.push(Move::promotion(source, target, Piece::Queen, false));
                generator.push(Move::promotion(source, target, Piece::Rook, false));
                generator.push(Move::promotion(source, target, Piece::Bishop, false));
                generator.push(Move::promotion(source, target, Piece::Knight, false));
                return;
            }

            let is_double_pawn = Self::is_double_pawn(source, target);
            if is_double_pawn {
                let mov = Move::double_pawn(source, target);
                generator.push(mov);
                return;
            } else {
                let mov = Move::quiet(source, target);
                generator.push(mov);
                return;
            }
        }
    }
}
