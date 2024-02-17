use crate::{
    bitboard::{square::Square, Bitboard},
    board::{piece::Piece, Board},
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
        _blockers: Bitboard,
    ) -> Bitboard {
        let mut pseudo_legals = Bitboard::default();

        let single_pushes = source.get_pawn_pushes(board.active());
        pseudo_legals |= single_pushes;

        // TODO: Add double pawn pushes

        let attacks = source.get_pawn_attacks(board.active());
        pseudo_legals |= attacks;

        pseudo_legals &= allowed;
        pseudo_legals
    }

    fn legals<T>(_generator: &mut NewMoveGenerator, _board: &Board)
    where
        T: CheckType,
    {
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
        let valid_knights = knights & unpinned;

        for source in valid_knights {
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

    fn legals<T>(_generator: &mut NewMoveGenerator, _board: &Board)
    where
        T: CheckType,
    {
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

    fn legals<T>(_generator: &mut NewMoveGenerator, _board: &Board)
    where
        T: CheckType,
    {
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

    fn legals<T>(_generator: &mut NewMoveGenerator, _board: &Board)
    where
        T: CheckType,
    {
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

    fn legals<T>(_generator: &mut NewMoveGenerator, _board: &Board)
    where
        T: CheckType,
    {
    }
}
