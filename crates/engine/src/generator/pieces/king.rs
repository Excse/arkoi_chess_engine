use base::{
    bitboard::Bitboard,
    board::{color::Color, Board},
    r#move::Move,
    square::{constants::*, Square},
};

use crate::generator::{CheckType, MoveGenerator, PieceGenerator};

pub(crate) struct KingGenerator;

impl PieceGenerator for KingGenerator {
    #[inline(always)]
    fn pseudo_legals(
        _board: &Board,
        source: Square,
        allowed: Bitboard,
        _blockers: Bitboard,
    ) -> Bitboard {
        let mut pseudo_legals = Bitboard::default();

        pseudo_legals |= source.get_king_moves();

        pseudo_legals &= allowed;
        pseudo_legals
    }

    #[inline(always)]
    fn legals<T>(generator: &mut MoveGenerator, board: &Board)
    where
        T: CheckType,
    {
        let king_square = board.get_king_square(board.active());
        let own_occupied = board.get_occupied(board.active());
        let all_occupied = board.get_all_occupied();
        let attacked = board.attacked();

        let allowed = !(own_occupied | attacked);

        // Generate all pseudo legal moves for the king.
        let moves = Self::pseudo_legals(board, king_square, allowed, all_occupied);

        for target in moves {
            let is_capture = board.get_piece_type(target).is_some();
            if is_capture {
                let mov = Move::capture(king_square, target);
                generator.push(mov);
            } else {
                let mov = Move::quiet(king_square, target);
                generator.push(mov);
            }
        }

        if !T::IN_CHECK {
            if board.active() == Color::White {
                if board.can_white_queenside() {
                    let mut nothing_inbetween = E1.get_between(A1);
                    nothing_inbetween &= all_occupied;
                    let nothing_inbetween = nothing_inbetween.is_empty();

                    let mut attacked_through_move = E1.get_between(C1);
                    attacked_through_move |= C1;
                    attacked_through_move &= attacked;
                    let attacked_through_move = !attacked_through_move.is_empty();

                    if nothing_inbetween && !attacked_through_move {
                        generator.push(Move::castle(E1, C1, false));
                    }
                }

                if board.can_white_kingside() {
                    let mut nothing_inbetween = E1.get_between(H1);
                    nothing_inbetween &= all_occupied;
                    let nothing_inbetween = nothing_inbetween.is_empty();

                    let mut attacked_through_move = E1.get_between(G1);
                    attacked_through_move |= G1;
                    attacked_through_move &= attacked;
                    let attacked_through_move = !attacked_through_move.is_empty();

                    if nothing_inbetween && !attacked_through_move {
                        generator.push(Move::castle(E1, G1, true));
                    }
                }
            } else if board.active() == Color::Black {
                if board.can_black_queenside() {
                    let mut nothing_inbetween = E8.get_between(A8);
                    nothing_inbetween &= all_occupied;
                    let nothing_inbetween = nothing_inbetween.is_empty();

                    let mut attacked_through_move = E8.get_between(C8);
                    attacked_through_move |= C8;
                    attacked_through_move &= attacked;
                    let attacked_through_move = !attacked_through_move.is_empty();

                    if nothing_inbetween && !attacked_through_move {
                        generator.push(Move::castle(E8, C8, false));
                    }
                }

                if board.can_black_kingside() {
                    let mut nothing_inbetween = E8.get_between(H8);
                    nothing_inbetween &= all_occupied;
                    let nothing_inbetween = nothing_inbetween.is_empty();

                    let mut attacked_through_move = E8.get_between(G8);
                    attacked_through_move |= G8;
                    attacked_through_move &= attacked;
                    let attacked_through_move = !attacked_through_move.is_empty();

                    if nothing_inbetween && !attacked_through_move {
                        generator.push(Move::castle(E8, G8, true));
                    }
                }
            }
        }
    }
}
