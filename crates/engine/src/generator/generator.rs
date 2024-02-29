use std::marker::PhantomData;

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

pub trait MoveType {
    const QUIET: bool;
    const CAPTURE: bool;
}

pub struct QuietMoves;

impl MoveType for QuietMoves {
    const QUIET: bool = true;
    const CAPTURE: bool = false;
}

pub struct CaptureMoves;

impl MoveType for CaptureMoves {
    const QUIET: bool = false;
    const CAPTURE: bool = true;
}

pub struct AllMoves;

impl MoveType for AllMoves {
    const QUIET: bool = true;
    const CAPTURE: bool = true;
}

pub(crate) trait PieceGenerator {
    fn pseudo_legals(
        board: &Board,
        source: Square,
        allowed: Bitboard,
        blockers: Bitboard,
    ) -> Bitboard;

    fn legals<C, M>(generator: &mut MoveGenerator<M>, board: &Board)
    where
        C: CheckType,
        M: MoveType;
}

pub struct MoveGenerator<M: MoveType> {
    moves: [Move; MAX_MOVES],
    size: usize,
    index: usize,
    phantom: PhantomData<M>,
}

impl<M: MoveType> MoveGenerator<M> {
    pub fn new(board: &Board) -> Self {
        let mut generator = Self {
            moves: [NULL_MOVE; MAX_MOVES],
            size: 0,
            index: 0,
            phantom: PhantomData,
        };

        let checkers = board.checkers();
        if checkers.is_empty() {
            PawnGenerator::legals::<NotInCheck, M>(&mut generator, board);
            KnightGenerator::legals::<NotInCheck, M>(&mut generator, board);
            BishopGenerator::legals::<NotInCheck, M>(&mut generator, board);
            RookGenerator::legals::<NotInCheck, M>(&mut generator, board);
            QueenGenerator::legals::<NotInCheck, M>(&mut generator, board);
            KingGenerator::legals::<NotInCheck, M>(&mut generator, board);
        } else if checkers.count_ones() == 1 {
            PawnGenerator::legals::<InCheck, M>(&mut generator, board);
            KnightGenerator::legals::<InCheck, M>(&mut generator, board);
            BishopGenerator::legals::<InCheck, M>(&mut generator, board);
            RookGenerator::legals::<InCheck, M>(&mut generator, board);
            QueenGenerator::legals::<InCheck, M>(&mut generator, board);
            KingGenerator::legals::<InCheck, M>(&mut generator, board);
        } else {
            KingGenerator::legals::<InCheck, M>(&mut generator, board);
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

impl<M: MoveType> ExactSizeIterator for MoveGenerator<M> {
    fn len(&self) -> usize {
        self.size
    }
}

impl<M: MoveType> Iterator for MoveGenerator<M> {
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
