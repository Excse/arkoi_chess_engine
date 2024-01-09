use std::fmt::Display;

use crate::{
    bitboard::Bitboard,
    board::{Board, BoardError, Piece},
    tables::{KING_MOVES, KNIGHT_MOVES, PAWN_ATTACKS, PAWN_PUSHES},
};

#[derive(Debug)]
pub struct Move {
    from: Bitboard,
    to: Bitboard,
}

impl Move {
    pub fn new(from: impl Into<Bitboard>, to: impl Into<Bitboard>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
        }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (from_rank, from_file) = self.from.get_rank_file();
        let (to_rank, to_file) = self.to.get_rank_file();
        write!(
            f,
            "From {}{} to {}{}",
            from_file, from_rank, to_file, to_rank
        )
    }
}

pub struct MoveGenerator {
    board: Board,
}

impl MoveGenerator {
    pub fn new(board: Board) -> Self {
        Self { board }
    }

    pub fn get_pseudo_moves(&self) -> Result<Vec<Move>, BoardError> {
        let mut moves = Vec::new();

        let pawn_moves = self.get_pawn_moves()?;
        moves.extend(pawn_moves);

        let king_moves = self.get_king_moves()?;
        moves.extend(king_moves);

        let knight_moves = self.get_knight_moves()?;
        moves.extend(knight_moves);

        Ok(moves)
    }

    fn get_king_moves(&self) -> Result<Vec<Move>, BoardError> {
        let mut moves = Vec::new();

        let mut kings = *self.board.get_active_piece_board(Piece::King);
        let own_unoccupied = !self.board.get_active();

        while kings.bits != 0 {
            let from_index = kings.bits.trailing_zeros() as usize;
            let from = Bitboard::index(from_index);
            kings ^= from;

            let moves_mask = KING_MOVES[from_index];
            let mut result = own_unoccupied & moves_mask;
            while result.bits != 0 {
                let to_index = result.bits.trailing_zeros() as usize;
                let to = Bitboard::index(to_index);
                result ^= to;

                moves.push(Move::new(from, to));
            }
        }

        Ok(moves)
    }

    fn get_knight_moves(&self) -> Result<Vec<Move>, BoardError> {
        let mut moves = Vec::new();

        let mut knights = *self.board.get_active_piece_board(Piece::Knight);
        let own_unoccupied = !self.board.get_active();

        while knights.bits != 0 {
            let from_index = knights.bits.trailing_zeros() as usize;
            let from = Bitboard::index(from_index);
            knights ^= from;

            let moves_mask = KNIGHT_MOVES[from_index];
            let mut result = own_unoccupied & moves_mask;
            while result.bits != 0 {
                let to_index = result.bits.trailing_zeros() as usize;
                let to = Bitboard::index(to_index);
                result ^= to;

                moves.push(Move::new(from, to));
            }
        }

        Ok(moves)
    }

    fn get_pawn_moves(&self) -> Result<Vec<Move>, BoardError> {
        let mut moves = Vec::new();

        let mut pawns = *self.board.get_active_piece_board(Piece::Pawn);
        let unactive_occupied = self.board.get_unactive();
        let active_index = self.board.active.index();
        let occupied = self.board.get_occupied();

        while pawns.bits != 0 {
            let from_index = pawns.bits.trailing_zeros() as usize;
            let from = Bitboard::index(from_index);
            pawns ^= from;

            let push_mask = PAWN_PUSHES[active_index][from_index];
            let result = occupied & push_mask;
            if result.bits == 0 {
                moves.push(Move::new(from, push_mask));

                let can_double_push = (Bitboard::RANK_2 & from) | (Bitboard::RANK_7 & from);
                if can_double_push.bits != 0 {
                    let from_index = push_mask.trailing_zeros() as usize;

                    let push_mask = PAWN_PUSHES[active_index][from_index];
                    let result = occupied & push_mask;
                    if result.bits == 0 {
                        moves.push(Move::new(from, push_mask));
                    }
                }
            }

            let attack_mask = PAWN_ATTACKS[active_index][from_index];
            let mut result = unactive_occupied & attack_mask;
            while result.bits != 0 {
                let to_index = result.bits.trailing_zeros() as usize;
                let to = Bitboard::index(to_index);
                result ^= to;

                moves.push(Move::new(from, to));
            }
        }

        Ok(moves)
    }
}
