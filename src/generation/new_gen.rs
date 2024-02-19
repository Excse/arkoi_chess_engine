use crate::{
    bitboard::{constants::*, square::Square, Bitboard},
    board::{color::Color, piece::Piece, Board, EnPassant},
};

use super::mov::Move;

pub const MAX_MOVES: usize = 256;

pub trait CheckType {
    const IN_CHECK: bool;
}

pub struct InCheck;

impl CheckType for InCheck {
    const IN_CHECK: bool = true;
}

pub struct NotInCheck;

impl CheckType for NotInCheck {
    const IN_CHECK: bool = false;
}

pub struct NewMoveGenerator {
    moves: [Move; MAX_MOVES],
    size: usize,
    index: usize,
}

impl NewMoveGenerator {
    pub fn new(board: &Board) -> Self {
        let mut generator = Self {
            moves: [Move::NULL_MOVE; MAX_MOVES],
            size: 0,
            index: 0,
        };

        let checkers = board.checkers();
        if checkers.is_empty() {
            PawnGenerator::legals::<NotInCheck>(&mut generator, board);
            KnightGenerator::legals::<NotInCheck>(&mut generator, board);
            BishopGenerator::legals::<NotInCheck>(&mut generator, board);
            RookGenerator::legals::<NotInCheck>(&mut generator, board);
            QueenGenerator::legals::<NotInCheck>(&mut generator, board);
            KingGenerator::legals::<NotInCheck>(&mut generator, board);
        } else if checkers.count_ones() == 1 {
            PawnGenerator::legals::<InCheck>(&mut generator, board);
            KnightGenerator::legals::<InCheck>(&mut generator, board);
            BishopGenerator::legals::<InCheck>(&mut generator, board);
            RookGenerator::legals::<InCheck>(&mut generator, board);
            QueenGenerator::legals::<InCheck>(&mut generator, board);
            KingGenerator::legals::<InCheck>(&mut generator, board);
        } else {
            KingGenerator::legals::<InCheck>(&mut generator, board);
        }

        generator
    }

    pub fn push(&mut self, mov: Move) {
        self.moves[self.size] = mov;
        self.size += 1;
    }
}

impl ExactSizeIterator for NewMoveGenerator {
    fn len(&self) -> usize {
        self.size
    }
}

impl Iterator for NewMoveGenerator {
    type Item = Move;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.len() {
            return None;
        } else {
            let mov = self.moves[self.index];
            self.index += 1;
            return Some(mov);
        }
    }
}

pub trait PieceGenerator {
    fn pseudo_legals(
        board: &Board,
        source: Square,
        allowed: Bitboard,
        blockers: Bitboard,
    ) -> Bitboard;

    fn legals<T>(generator: &mut NewMoveGenerator, board: &Board)
    where
        T: CheckType;
}

pub struct PawnGenerator;

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
    fn legals<T>(generator: &mut NewMoveGenerator, board: &Board)
    where
        T: CheckType,
    {
        let pawns = board.get_piece_board(board.active(), Piece::Pawn);
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

        // At first calculate every pawn move that is not pinned.
        for source in pawns & unpinned {
            // Now we can generate all pseudo legal moves for the pawn and
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

                // TODO: Quick and dirty, needs to be refactored.
                let rank = target.rank();
                let is_promotion = rank == 0 || rank == 7;
                if is_promotion {
                    generator.push(Move::promotion(
                        source,
                        target,
                        Piece::Queen,
                        captured_piece,
                    ));
                    generator.push(Move::promotion(source, target, Piece::Rook, captured_piece));
                    generator.push(Move::promotion(
                        source,
                        target,
                        Piece::Bishop,
                        captured_piece,
                    ));
                    generator.push(Move::promotion(
                        source,
                        target,
                        Piece::Knight,
                        captured_piece,
                    ));
                } else {
                    // Create a potential capture move. At the end it doesn't matter if
                    // the captured square is set or not.
                    let mov = Move::capture(Piece::Pawn, source, target, captured_piece);
                    generator.push(mov);
                }
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
                    // If there is a piece on the target square, we capture it.
                    let captured_piece = match board.get_piece_type(target) {
                        Some(colored_piece) => colored_piece.piece,
                        None => Piece::None,
                    };

                    // TODO: Quick and dirty, needs to be refactored.
                    let rank = target.rank();
                    let is_promotion = rank == 0 || rank == 7;
                    if is_promotion {
                        generator.push(Move::promotion(
                            source,
                            target,
                            Piece::Queen,
                            captured_piece,
                        ));
                        generator.push(Move::promotion(
                            source,
                            target,
                            Piece::Rook,
                            captured_piece,
                        ));
                        generator.push(Move::promotion(
                            source,
                            target,
                            Piece::Bishop,
                            captured_piece,
                        ));
                        generator.push(Move::promotion(
                            source,
                            target,
                            Piece::Knight,
                            captured_piece,
                        ));
                    } else {
                        // Create a potential capture move. At the end it doesn't matter if
                        // the captured square is set or not.
                        let mov = Move::capture(Piece::Pawn, source, target, captured_piece);
                        generator.push(mov);
                    }
                }
            }
        }

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

                let mov = Move::en_passant(source, destination, to_capture);
                generator.push(mov);
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
}

pub struct KnightGenerator;

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
    fn legals<T>(generator: &mut NewMoveGenerator, board: &Board)
    where
        T: CheckType,
    {
        let knights = board.get_piece_board(board.active(), Piece::Knight);
        let own_occupied = board.get_occupied(board.active());
        let all_occupied = board.get_all_occupied();
        let unpinned = !board.pinned();

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

        // The knight can never move on a pin ray because of it's movement.
        // Thus we can ignore every knight that is pinned.
        for source in knights & unpinned {
            // Now we can generate all pseudo legal moves for the knight and
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
                let mov = Move::capture(Piece::Knight, source, target, captured_piece);
                generator.push(mov);
            }
        }
    }
}

pub struct RookGenerator;

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
    fn legals<T>(generator: &mut NewMoveGenerator, board: &Board)
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

pub struct BishopGenerator;

impl PieceGenerator for BishopGenerator {
    #[inline(always)]
    fn pseudo_legals(
        _board: &Board,
        source: Square,
        allowed: Bitboard,
        blockers: Bitboard,
    ) -> Bitboard {
        let mut pseudo_legals = Bitboard::default();

        pseudo_legals |= source.get_bishop_attacks(blockers);

        pseudo_legals &= allowed;
        pseudo_legals
    }

    #[inline(always)]
    fn legals<T>(generator: &mut NewMoveGenerator, board: &Board)
    where
        T: CheckType,
    {
        let bishops = board.get_piece_board(board.active(), Piece::Bishop);
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

        // At first calculate every bishop move that is not pinned.
        for source in bishops & unpinned {
            // Now we can generate all pseudo legal moves for the bishop and
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
                let mov = Move::capture(Piece::Bishop, source, target, captured_piece);
                generator.push(mov);
            }
        }

        // It is not possible to move pinned pieces when in check
        if !T::IN_CHECK {
            let king_square = board.get_king_square(board.active());

            // If not in check we calculate every move for pinned pieces
            for source in bishops & pinned {
                // The line of the bishop to the king.
                let line = king_square.get_line(source);
                // We just can move on the line. This will allow us to generate
                // every move between the pinner and the king, but also the capture
                // move of the pinner without interfering with the other pinned lines.
                let allowed = allowed & line;

                // Now we can generate all pseudo legal moves for the bishop and
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
                    let mov = Move::capture(Piece::Bishop, source, target, captured_piece);
                    generator.push(mov);
                }
            }
        }
    }
}

pub struct QueenGenerator;

impl PieceGenerator for QueenGenerator {
    #[inline(always)]
    fn pseudo_legals(
        _board: &Board,
        source: Square,
        allowed: Bitboard,
        blockers: Bitboard,
    ) -> Bitboard {
        let mut pseudo_legals = Bitboard::default();

        pseudo_legals |= source.get_rook_attacks(blockers);
        pseudo_legals |= source.get_bishop_attacks(blockers);

        pseudo_legals &= allowed;
        pseudo_legals
    }

    #[inline(always)]
    fn legals<T>(generator: &mut NewMoveGenerator, board: &Board)
    where
        T: CheckType,
    {
        let queens = board.get_piece_board(board.active(), Piece::Queen);
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

        // At first calculate every queen move that is not pinned.
        for source in queens & unpinned {
            // Now we can generate all pseudo legal moves for the queen and
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
                let mov = Move::capture(Piece::Queen, source, target, captured_piece);
                generator.push(mov);
            }
        }

        // It is not possible to move pinned pieces when in check
        if !T::IN_CHECK {
            let king_square = board.get_king_square(board.active());

            // If not in check we calculate every move for pinned pieces
            for source in queens & pinned {
                // The line of the queen to the king.
                let line = king_square.get_line(source);
                // We just can move on the line. This will allow us to generate
                // every move between the pinner and the king, but also the capture
                // move of the pinner without interfering with the other pinned lines.
                let allowed = allowed & line;

                // Now we can generate all pseudo legal moves for the queen and
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
                    let mov = Move::capture(Piece::Queen, source, target, captured_piece);
                    generator.push(mov);
                }
            }
        }
    }
}

pub struct KingGenerator;

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
    fn legals<T>(generator: &mut NewMoveGenerator, board: &Board)
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
            // If there is a piece on the target square, we capture it.
            let captured_piece = match board.get_piece_type(target) {
                Some(colored_piece) => colored_piece.piece,
                None => Piece::None,
            };

            // Create a potential capture move. At the end it doesn't matter if
            // the captured square is set or not.
            let mov = Move::capture(Piece::King, king_square, target, captured_piece);
            generator.push(mov);
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
                        generator.push(Move::castling(E1, C1));
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
                        generator.push(Move::castling(E1, G1));
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
                        generator.push(Move::castling(E8, C8));
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
                        generator.push(Move::castling(E8, G8));
                    }
                }
            }
        }
    }
}
