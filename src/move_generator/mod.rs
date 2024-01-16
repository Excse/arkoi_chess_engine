pub mod error;

use std::{fmt::Display, str::FromStr};

use crate::{
    bitboard::{Bitboard, Square},
    board::{Board, Piece},
    tables::{KING_MOVES, KNIGHT_MOVES, PAWN_ATTACKS, PAWN_PUSHES, RAYS},
};

use self::error::{InvalidMoveFormat, MoveError, PieceNotFound};

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

    pub fn parse(input: String, board: &Board) -> Result<Self, MoveError> {
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
    pub fn get_legal_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::new();

        // Get all king moves that are not illegal
        {
            let mut other_board = board.clone();
            other_board.swap_active();

            // All moves from the opponent where the king cant move to
            let mut other_moves = Vec::new();

            // Get all sliding moves from the opponent without the king of the current player
            let own_kings = board.get_active_squares_by_piece(Piece::King);
            for square in own_kings.clone() {
                other_board.toggle_other(Piece::King, square);
            }
            let other_sliding_moves = self.get_sliding_moves(&other_board);
            other_moves.extend(other_sliding_moves);
            for square in own_kings {
                other_board.toggle_other(Piece::King, square);
            }

            let other_non_sliding_moves = self.get_non_sliding_moves(&other_board);
            other_moves.extend(other_non_sliding_moves);

            let mut forbidden_bb = Bitboard::default();
            for mov in other_moves {
                forbidden_bb |= mov.to;
            }

            println!("Forbidden:\n{}", forbidden_bb);

            let king_moves = self.get_king_moves(board, forbidden_bb);
            moves.extend(king_moves);
        }

        // let pawn_moves = self.get_pawn_moves(board);
        // moves.extend(pawn_moves);

        // let knight_moves = self.get_knight_moves(board);
        // moves.extend(knight_moves);

        // let bishop_moves = self.get_bishop_moves(board);
        // moves.extend(bishop_moves);

        // let rook_moves = self.get_rook_moves(board);
        // moves.extend(rook_moves);

        // let queen_moves = self.get_queen_moves(board);
        // moves.extend(queen_moves);

        moves
    }

    pub fn get_non_sliding_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::new();

        let pawn_moves = self.get_pawn_moves(board);
        moves.extend(pawn_moves);

        let king_moves = self.get_king_moves(board, Bitboard::default());
        moves.extend(king_moves);

        let knight_moves = self.get_knight_moves(board);
        moves.extend(knight_moves);

        moves
    }

    pub fn get_sliding_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::new();

        let bishop_moves = self.get_bishop_moves(board);
        moves.extend(bishop_moves);

        let rook_moves = self.get_rook_moves(board);
        moves.extend(rook_moves);

        let queen_moves = self.get_queen_moves(board);
        moves.extend(queen_moves);

        moves
    }

    fn get_king_moves(&self, board: &Board, forbidden: Bitboard) -> Vec<Move> {
        let mut moves = Vec::new();

        let squares = board.get_active_squares_by_piece(Piece::King);
        println!("Squares: {:?}", squares);
        for from in squares {
            let index = from.index as usize;

            let moves_bb = self.get_single_king_moves(board, index);
            let moves_bb = moves_bb & !forbidden;

            let extracted = self.extract_moves(Piece::King, board, from, moves_bb);
            moves.extend(extracted);
        }
        print!(" - Moves: ");
        for mov in moves.iter() {
            print!("{}, ", mov);
        }
        println!();

        moves
    }

    fn get_single_king_moves(&self, board: &Board, index: usize) -> Bitboard {
        let own_unoccupied = !board.get_own_occupied();

        let moves_mask = KING_MOVES[index];
        own_unoccupied & moves_mask
    }

    fn get_knight_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::new();

        let squares = board.get_active_squares_by_piece(Piece::Knight);
        for from in squares {
            let index = from.index as usize;

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

        let squares = board.get_active_squares_by_piece(Piece::Pawn);
        for from in squares {
            let from_bb: Bitboard = from.into();
            let index = from.index as usize;

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

        let squares = board.get_active_squares_by_piece(Piece::Bishop);
        for from in squares {
            let index = from.index as usize;

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

        let squares = board.get_active_squares_by_piece(Piece::Rook);
        for from in squares {
            let index = from.index as usize;

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

        let squares = board.get_active_squares_by_piece(Piece::Queen);
        for from in squares {
            let index = from.index as usize;

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
