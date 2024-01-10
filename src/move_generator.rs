use std::fmt::Display;

use crate::{
    bitboard::Bitboard,
    board::{Board, BoardError, Piece},
    tables::{KING_MOVES, KNIGHT_MOVES, PAWN_ATTACKS, PAWN_PUSHES},
};

#[derive(Debug)]
pub struct Move {
    pub piece: Piece,
    pub attack: bool,
    pub from: Bitboard,
    pub to: Bitboard,
}

impl Move {
    pub fn new(
        piece: Piece,
        attack: bool,
        from: impl Into<Bitboard>,
        to: impl Into<Bitboard>,
    ) -> Self {
        Self {
            piece,
            attack,
            from: from.into(),
            to: to.into(),
        }
    }

    // TODO: Implement error handling and also make it much safer e.g. from_rank
    pub fn from_str(input: String, board: &Board) -> Result<Self, ()> {
        let mut input = input.chars();

        let from_file = input.next().ok_or(())? as u8;
        let from_file = from_file - b'a';
        let from_rank = input.next().ok_or(())? as u8;
        let from_rank = from_rank - b'0' - 1;
        let from = Bitboard::square(from_rank, from_file);

        let piece = board.get_piece_type(board.active, from).ok_or(())?;

        let to_file = input.next().ok_or(())? as u8;
        let to_file = to_file - b'a';
        let to_rank = input.next().ok_or(())? as u8;
        let to_rank = to_rank - b'0' - 1;
        let to = Bitboard::square(to_rank, to_file);

        let attacked = board.get_piece_type(!board.active, to);

        Ok(Self::new(piece, attacked.is_some(), from, to))
    }

    pub fn toggle(piece: Piece, square: impl Into<Bitboard>) -> Self {
        Self {
            piece,
            attack: false,
            from: Bitboard::default(),
            to: square.into(),
        }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (from_rank, from_file) = self.from.get_rank_file();
        let (to_rank, to_file) = self.to.get_rank_file();
        write!(f, "{}{}{}{}", from_file, from_rank, to_file, to_rank)
    }
}

pub struct MoveGenerator<'a> {
    board: &'a Board,
}

impl<'a> MoveGenerator<'a> {
    pub fn new(board: &'a Board) -> Self {
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
        let other_occupied = self.board.get_unactive();

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

                let is_attack = other_occupied & to;
                let is_attack = is_attack.bits != 0;

                let mov = Move::new(Piece::King, is_attack, from, to);
                moves.push(mov);
            }
        }

        Ok(moves)
    }

    fn get_knight_moves(&self) -> Result<Vec<Move>, BoardError> {
        let mut moves = Vec::new();

        let mut knights = *self.board.get_active_piece_board(Piece::Knight);
        let own_unoccupied = !self.board.get_active();
        let other_occupied = self.board.get_unactive();

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

                let is_attack = other_occupied & to;
                let is_attack = is_attack.bits != 0;

                let mov = Move::new(Piece::Knight, is_attack, from, to);
                moves.push(mov);
            }
        }

        Ok(moves)
    }

    fn get_pawn_moves(&self) -> Result<Vec<Move>, BoardError> {
        let mut moves = Vec::new();

        let mut pawns = *self.board.get_active_piece_board(Piece::Pawn);
        let other_occupied = self.board.get_unactive();
        let active_index = self.board.active.index();
        let occupied = self.board.get_occupied();

        while pawns.bits != 0 {
            let from_index = pawns.bits.trailing_zeros() as usize;
            let from = Bitboard::index(from_index);
            pawns ^= from;

            let push_mask = PAWN_PUSHES[active_index][from_index];
            let result = occupied & push_mask;
            if result.bits == 0 {
                let mov = Move::new(Piece::Pawn, false, from, push_mask);
                moves.push(mov);

                let can_double_push = (Bitboard::RANK_2 & from) | (Bitboard::RANK_7 & from);
                if can_double_push.bits != 0 {
                    let from_index = push_mask.trailing_zeros() as usize;

                    let push_mask = PAWN_PUSHES[active_index][from_index];
                    let result = occupied & push_mask;
                    if result.bits == 0 {
                        let mov = Move::new(Piece::Pawn, false, from, push_mask);
                        moves.push(mov);
                    }
                }
            }

            let attack_mask = PAWN_ATTACKS[active_index][from_index];
            let mut result = other_occupied & attack_mask;
            while result.bits != 0 {
                let to_index = result.bits.trailing_zeros() as usize;
                let to = Bitboard::index(to_index);
                result ^= to;

                let mov = Move::new(Piece::Pawn, true, from, to);
                moves.push(mov);
            }
        }

        Ok(moves)
    }
}
