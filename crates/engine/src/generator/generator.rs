use base::{
    bitboard::Bitboard,
    board::Board,
    r#move::{constants::NULL_MOVE, Move},
    square::Square,
};

use super::pieces::{
    BishopGenerator, KingGenerator, KnightGenerator, PawnGenerator, QueenGenerator, RookGenerator,
};

pub const MAX_MOVES: usize = 256;

pub(crate) trait CheckType {
    const IN_CHECK: bool;
}

pub(crate) struct InCheck;

impl CheckType for InCheck {
    const IN_CHECK: bool = true;
}

pub(crate) struct NotInCheck;

impl CheckType for NotInCheck {
    const IN_CHECK: bool = false;
}

pub(crate) trait PieceGenerator {
    fn pseudo_legals(
        board: &Board,
        source: Square,
        allowed: Bitboard,
        blockers: Bitboard,
    ) -> Bitboard;

    fn legals<T>(generator: &mut MoveGenerator, board: &Board)
    where
        T: CheckType;
}

pub struct MoveGenerator {
    moves: [Move; MAX_MOVES],
    size: usize,
    index: usize,
}

impl MoveGenerator {
    pub fn new(board: &Board) -> Self {
        let mut generator = Self {
            moves: [NULL_MOVE; MAX_MOVES],
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

    // TODO: Do this in a better way.
    #[inline(always)]
    pub fn is_checkmate(&self, board: &Board) -> bool {
        self.size == 0 && board.is_check()
    }

    // TODO: Do this in a better way.
    #[inline(always)]
    pub fn is_stalemate(&self, board: &Board) -> bool {
        self.size == 0 && !board.is_check()
    }
}

impl ExactSizeIterator for MoveGenerator {
    fn len(&self) -> usize {
        self.size
    }
}

impl Iterator for MoveGenerator {
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
