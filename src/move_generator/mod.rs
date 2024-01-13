mod error;

use std::{fmt::Display, str::FromStr};

use crate::{
    bitboard::{Bitboard, Square},
    board::{Board, Piece},
    tables::{KING_MOVES, KNIGHT_MOVES, PAWN_ATTACKS, PAWN_PUSHES, RAYS},
};

use self::error::{InvalidMoveFormat, PieceNotFound, Result};

#[derive(Debug)]
pub struct Move {
    pub piece: Piece,
    pub attack: bool,
    pub from: Square,
    pub to: Square,
}

impl Move {
    pub fn new(piece: Piece, attack: bool, from: Square, to: Square) -> Self {
        Self {
            piece,
            attack,
            from,
            to,
        }
    }

    pub fn parse(input: String, board: &Board) -> Result<Self> {
        if input.len() != 4 {
            return Err(InvalidMoveFormat::new(input.clone()).into());
        }

        let from = Square::from_str(&input[0..2])?;
        let to = Square::from_str(&input[2..4])?;

        let piece = board
            .get_piece_type(board.active, from)
            .ok_or(PieceNotFound::new(from.to_string()))?;
        let attacked = board.get_piece_type(!board.active, to);

        Ok(Self::new(piece, attacked.is_some(), from, to))
    }

    pub fn toggle(piece: Piece, to: Square) -> Self {
        Self {
            piece,
            attack: false,
            from: Square::default(),
            to,
        }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let from = self.from.to_string();
        let to = self.to.to_string();
        write!(f, "{}{}", from, to)
    }
}

#[derive(Default)]
pub struct MoveGenerator;

impl MoveGenerator {
    pub fn get_pseudo_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::new();

        let pawn_moves = self.get_pawn_moves(board);
        moves.extend(pawn_moves);

        let king_moves = self.get_king_moves(board);
        moves.extend(king_moves);

        let knight_moves = self.get_knight_moves(board);
        moves.extend(knight_moves);

        let bishop_moves = self.get_bishop_moves(board);
        moves.extend(bishop_moves);

        let rook_moves = self.get_rook_moves(board);
        moves.extend(rook_moves);

        let queen_moves = self.get_queen_moves(board);
        moves.extend(queen_moves);

        moves
    }

    fn get_king_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::new();

        let mut kings = *board.get_active_piece_board(Piece::King);
        while kings.bits != 0 {
            let index = kings.bits.trailing_zeros() as usize;
            let from = Square::index(index as u8);
            kings ^= from;

            let moves_bb = self.get_single_king_moves(board, index);
            let extracted = self.extract_moves(Piece::King, board, from, moves_bb);
            moves.extend(extracted);
        }

        moves
    }

    fn get_single_king_moves(&self, board: &Board, index: usize) -> Bitboard {
        let own_unoccupied = !board.get_own_occupied();

        let moves_mask = KING_MOVES[index];
        own_unoccupied & moves_mask
    }

    fn get_knight_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::new();

        let mut knights = *board.get_active_piece_board(Piece::Knight);
        while knights.bits != 0 {
            let index = knights.bits.trailing_zeros() as usize;
            let from = Square::index(index as u8);
            knights ^= from;

            let moves_bb = self.get_single_knight_moves(board, index);
            let extracted = self.extract_moves(Piece::Knight, board, from, moves_bb);
            moves.extend(extracted);
        }

        moves
    }

    fn get_single_knight_moves(&self, board: &Board, index: usize) -> Bitboard {
        let own_unoccupied = !board.get_own_occupied();

        let moves_mask = KNIGHT_MOVES[index];
        own_unoccupied & moves_mask
    }

    fn get_pawn_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::new();

        let mut pawns = *board.get_active_piece_board(Piece::Pawn);
        while pawns.bits != 0 {
            let index = pawns.bits.trailing_zeros() as usize;
            let from = Square::index(index as u8);
            let from_bb: Bitboard = from.into();
            pawns ^= from_bb;

            let moves_bb = self.get_single_pawn_moves(board, index, from_bb);
            let extracted = self.extract_moves(Piece::Pawn, board, from, moves_bb);
            moves.extend(extracted);

            let moves_bb = self.get_single_pawn_attacks(board, index);
            let extracted = self.extract_moves(Piece::Pawn, board, from, moves_bb);
            moves.extend(extracted);
        }

        moves
    }

    fn get_single_pawn_moves(&self, board: &Board, index: usize, from_bb: Bitboard) -> Bitboard {
        let active_index = board.active.index();
        let all_occupied = board.get_all_occupied();

        let push_mask = PAWN_PUSHES[active_index][index];
        let attacking = all_occupied & push_mask;
        if attacking.bits != 0 {
            return Bitboard::default();
        }

        let mut moves = Bitboard::bits(push_mask);

        let double_push_allowed = (Bitboard::RANK_2 & from_bb) | (Bitboard::RANK_7 & from_bb);
        if double_push_allowed.bits == 0 {
            return moves;
        }

        let index = push_mask.trailing_zeros() as usize;

        let push_mask = PAWN_PUSHES[active_index][index];
        let attacking = all_occupied & push_mask;
        if attacking.bits != 0 {
            return moves;
        }

        moves |= push_mask;
        moves
    }

    fn get_single_pawn_attacks(&self, board: &Board, index: usize) -> Bitboard {
        let other_occupied = board.get_other_occpuied();
        let active_index = board.active.index();

        let attack_mask = PAWN_ATTACKS[active_index][index];
        other_occupied & attack_mask
    }

    fn get_bishop_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::new();

        let mut bishops = *board.get_active_piece_board(Piece::Bishop);
        while bishops.bits != 0 {
            let index = bishops.bits.trailing_zeros() as usize;
            let from = Square::index(index as u8);
            bishops ^= from;

            let moves_bb = self.get_single_bishop_moves(board, index);
            let extracted = self.extract_moves(Piece::Bishop, board, from, moves_bb);
            moves.extend(extracted);
        }

        moves
    }

    fn get_single_bishop_moves(&self, board: &Board, index: usize) -> Bitboard {
        let own_occupied = board.get_own_occupied();
        let all_occupied = board.get_all_occupied();
        let mut moves = Bitboard::default();

        let north_east_ray = RAYS[index][2];
        moves |= north_east_ray;
        let blocking = north_east_ray & all_occupied;
        if blocking.bits != 0 {
            let blocker_index = blocking.get_trailing_index();
            moves &= !RAYS[blocker_index][2];
        }

        let south_east_ray = RAYS[index][7];
        moves |= south_east_ray;
        let blocking = south_east_ray & all_occupied;
        if blocking.bits != 0 {
            let blocker_index = blocking.get_leading_index();
            moves &= !RAYS[blocker_index][7];
        }

        let south_west_ray = RAYS[index][5];
        moves |= south_west_ray;
        let blocking = south_west_ray & all_occupied;
        if blocking.bits != 0 {
            let blocker_index = blocking.get_leading_index();
            moves &= !RAYS[blocker_index][5];
        }

        let north_west_ray = RAYS[index][0];
        moves |= north_west_ray;
        let blocking = north_west_ray & all_occupied;
        if blocking.bits != 0 {
            let blocker_index = blocking.get_trailing_index();
            moves &= !RAYS[blocker_index][0];
        }

        moves ^= own_occupied & moves;
        moves
    }

    fn get_rook_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::new();

        let mut rooks = *board.get_active_piece_board(Piece::Rook);
        while rooks.bits != 0 {
            let index = rooks.bits.trailing_zeros() as usize;
            let from = Square::index(index as u8);
            rooks ^= from;

            let moves_bb = self.get_single_rook_moves(board, index);
            let extracted = self.extract_moves(Piece::Rook, board, from, moves_bb);
            moves.extend(extracted);
        }

        moves
    }

    fn get_single_rook_moves(&self, board: &Board, index: usize) -> Bitboard {
        let own_occupied = board.get_own_occupied();
        let all_occupied = board.get_all_occupied();
        let mut moves = Bitboard::default();

        let north_ray = RAYS[index][1];
        moves |= north_ray;
        let blocking = north_ray & all_occupied;
        if blocking.bits != 0 {
            let blocker_index = blocking.get_trailing_index();
            moves &= !RAYS[blocker_index][1];
        }

        let east_ray = RAYS[index][4];
        moves |= east_ray;
        let blocking = east_ray & all_occupied;
        if blocking.bits != 0 {
            let blocker_index = blocking.get_trailing_index();
            moves &= !RAYS[blocker_index][4];
        }

        let south_ray = RAYS[index][6];
        moves |= south_ray;
        let blocking = south_ray & all_occupied;
        if blocking.bits != 0 {
            let blocker_index = blocking.get_leading_index();
            moves &= !RAYS[blocker_index][6];
        }

        let west_ray = RAYS[index][3];
        moves |= west_ray;
        let blocking = west_ray & all_occupied;
        if blocking.bits != 0 {
            let blocker_index = blocking.get_leading_index();
            moves &= !RAYS[blocker_index][3];
        }

        moves ^= own_occupied & moves;
        moves
    }

    fn get_queen_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::new();

        let mut queens = *board.get_active_piece_board(Piece::Queen);
        while queens.bits != 0 {
            let index = queens.bits.trailing_zeros() as usize;
            let from = Square::index(index as u8);
            queens ^= from;

            let bishop_bb = self.get_single_bishop_moves(board, index);
            let rook_bb = self.get_single_rook_moves(board, index);
            let moves_bb = bishop_bb | rook_bb;
            let extracted = self.extract_moves(Piece::Queen, board, from, moves_bb);
            moves.extend(extracted);
        }

        moves
    }

    fn extract_moves(
        &self,
        piece: Piece,
        board: &Board,
        from: Square,
        mut moves_bb: Bitboard,
    ) -> Vec<Move> {
        let mut moves = Vec::new();

        let other_occupied = board.get_other_occpuied();
        while moves_bb.bits != 0 {
            let index = moves_bb.bits.trailing_zeros() as usize;
            let to = Square::index(index as u8);
            moves_bb ^= to;

            let is_attack = other_occupied & to;
            let is_attack = is_attack.bits != 0;

            let mov = Move::new(piece, is_attack, from, to);
            moves.push(mov);
        }

        moves
    }
}
