pub mod error;

use std::{cmp::Ordering, fmt::Display, str::FromStr};

use crate::{
    bitboard::{Bitboard, Square},
    board::{Board, Piece},
    lookup::tables::{
        BETWEEN_LOOKUP, COMBINED_BISHOP_RAYS, COMBINED_ROOK_RAYS, KING_MOVES, KNIGHT_MOVES,
        PAWN_ATTACKS, PAWN_PUSHES, RAYS,
    },
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
        let own_kings = board.get_active_squares_by_piece(Piece::King);

        let mut checked_king = None;
        for king in own_kings.clone() {
            let checkers = self.get_checkers(board, king);
            if checkers.is_empty() {
                continue;
            }

            // TODO: Not true if the attacker can be removed
            if checked_king.is_some() {
                // Checkmated in some fun gamemode
                // TODO: Throw error
                return Vec::new();
            }

            checked_king = Some((king, checkers));
        }

        match checked_king {
            Some((king, checkers)) => {
                if checkers.len() == 1 {
                    self.get_single_check_moves(king, checkers[0], board)
                } else {
                    self.get_double_check_moves(king, board)
                }
            }
            None => self.get_unchecked_moves(board),
        }
    }

    // TODO: Handle unwrap
    pub fn get_single_check_moves(
        &self,
        king: Square,
        checker: Square,
        board: &Board,
    ) -> Vec<Move> {
        let mut moves = Vec::new();

        let attacker_direction = self.get_direction_index(checker, king).unwrap();
        let mut attacker_ray = RAYS[checker.index][attacker_direction];
        attacker_ray &= !RAYS[king.index][attacker_direction];

        let unfiltered_moves = self.get_unchecked_moves(board);
        for mov in unfiltered_moves {
            let is_blocking = mov.to & attacker_ray;
            if mov.attack && mov.to == checker {
                // Capture the checking piece
                println!("Capture the checking piece: {}", mov);
                moves.push(mov);
            } else if mov.piece == Piece::King {
                // Move the king out of check
                println!("Move the king out of check: {}", mov);
                moves.push(mov);
            } else if is_blocking.bits != 0 {
                // Move a piece between the checking piece and the king
                println!(
                    "Move a piece between the checking piece and the king: {}",
                    mov
                );
                moves.push(mov);
            }
        }

        moves
    }

    pub fn get_direction_index(&self, from: Square, to: Square) -> Option<usize> {
        let rank_cmp = to.rank.cmp(&from.rank);
        let file_cmp = to.file.cmp(&from.file);
        if rank_cmp.is_eq() && file_cmp.is_eq() {
            return None;
        }

        let rank_diff = to.rank as i8 - from.rank as i8;
        let file_diff = to.file as i8 - from.file as i8;
        let equal_delta = rank_diff.abs() == file_diff.abs();

        return Some(match (rank_cmp, file_cmp, equal_delta) {
            (Ordering::Greater, Ordering::Less, true) => 0,
            (Ordering::Greater, Ordering::Equal, false) => 1,
            (Ordering::Greater, Ordering::Greater, true) => 2,

            (Ordering::Equal, Ordering::Less, false) => 3,
            (Ordering::Equal, Ordering::Greater, false) => 4,

            (Ordering::Less, Ordering::Less, true) => 5,
            (Ordering::Less, Ordering::Equal, false) => 6,
            (Ordering::Less, Ordering::Greater, true) => 7,

            _ => return None,
        });
    }

    pub fn get_double_check_moves(&self, square: Square, board: &Board) -> Vec<Move> {
        self.get_legal_king_moves(square, board)
    }

    // TODO: Clean up the code & fix bugs, this is a mess
    pub fn get_unchecked_moves(&self, board: &Board) -> Vec<Move> {
        let own_kings = board.get_active_squares_by_piece(Piece::King);

        let mut pinned_squares = Vec::new();
        for king in &own_kings {
            let mut other_board = board.clone();
            other_board.swap_active();

            let king_bishop_ray = COMBINED_BISHOP_RAYS[king.index];
            let other_bishops = board.get_other_squares_by_piece(Piece::Bishop);
            for bishop in other_bishops {
                let bishop_ray = COMBINED_BISHOP_RAYS[bishop.index];
                let mut pinned = bishop_ray & king_bishop_ray;
                pinned &= BETWEEN_LOOKUP[king.index][bishop.index];
                let pinned = self.extract_squares(Bitboard::bits(pinned));
                pinned_squares.extend(pinned);
            }

            let king_rook_ray = COMBINED_ROOK_RAYS[king.index];
            let other_rooks = board.get_other_squares_by_piece(Piece::Rook);
            for rook in other_rooks {
                let rook_ray = COMBINED_ROOK_RAYS[rook.index];
                let mut pinned = rook_ray & king_rook_ray;
                pinned &= BETWEEN_LOOKUP[king.index][rook.index];
                let pinned = self.extract_squares(Bitboard::bits(pinned));
                pinned_squares.extend(pinned);
            }

            let king_queen_ray = COMBINED_BISHOP_RAYS[king.index] | COMBINED_ROOK_RAYS[king.index];
            let other_queens = board.get_other_squares_by_piece(Piece::Queen);
            for queen in other_queens {
                let queen_ray = COMBINED_BISHOP_RAYS[queen.index] | COMBINED_ROOK_RAYS[queen.index];
                let mut pinned = queen_ray & king_queen_ray;
                pinned &= BETWEEN_LOOKUP[king.index][queen.index];
                let pinned = self.extract_squares(Bitboard::bits(pinned));
                pinned_squares.extend(pinned);
            }
        }

        let mut moves = Vec::new();

        let pawn_moves = self.get_pawn_moves(board);
        for mov in pawn_moves {
            if pinned_squares.contains(&mov.from) {
                println!("Pinned pawn: {}", mov);
                continue;
            }

            moves.push(mov);
        }

        let knight_moves = self.get_knight_moves(board);
        for mov in knight_moves {
            if pinned_squares.contains(&mov.from) {
                continue;
            }

            moves.push(mov);
        }

        let bishop_moves = self.get_bishop_moves(board);
        for mov in bishop_moves {
            if pinned_squares.contains(&mov.from) {
                continue;
            }

            moves.push(mov);
        }

        let rook_moves = self.get_rook_moves(board);
        for mov in rook_moves {
            if pinned_squares.contains(&mov.from) {
                continue;
            }

            moves.push(mov);
        }

        let queen_moves = self.get_queen_moves(board);
        for mov in queen_moves {
            if pinned_squares.contains(&mov.from) {
                continue;
            }

            moves.push(mov);
        }

        for king in own_kings {
            let king_moves = self.get_legal_king_moves(king, board);
            moves.extend(king_moves);
        }

        moves
    }

    pub fn get_checkers(&self, board: &Board, square: Square) -> Vec<Square> {
        let mut attackers = Vec::new();

        let index = square.index as usize;

        let self_pawn_moves = self.get_single_pawn_attacks(board, index);
        let other_pawns = board.get_other_piece_board(Piece::Pawn);
        let pawn_attackers = self_pawn_moves & other_pawns;
        if pawn_attackers.bits != 0 {
            let pawn_attackers = self.extract_squares(pawn_attackers);
            attackers.extend(pawn_attackers);
        }

        let self_knight_moves = self.get_single_knight_moves(board, index);
        let other_knights = board.get_other_piece_board(Piece::Knight);
        let knight_attackers = self_knight_moves & other_knights;
        if knight_attackers.bits != 0 {
            let knight_attackers = self.extract_squares(knight_attackers);
            attackers.extend(knight_attackers);
        }

        let self_bitshop_moves = self.get_single_bishop_moves(board, index);
        let other_bishops = board.get_other_piece_board(Piece::Bishop);
        let bishop_attackers = self_bitshop_moves & other_bishops;
        if bishop_attackers.bits != 0 {
            let bishop_attackers = self.extract_squares(bishop_attackers);
            attackers.extend(bishop_attackers);
        }

        let self_rook_moves = self.get_single_rook_moves(board, index);
        let other_rooks = board.get_other_piece_board(Piece::Rook);
        let rook_attackers = self_rook_moves & other_rooks;
        if rook_attackers.bits != 0 {
            let rook_attackers = self.extract_squares(rook_attackers);
            attackers.extend(rook_attackers);
        }

        let self_queen_moves = self.get_single_bishop_moves(board, index);
        let other_queens = board.get_other_piece_board(Piece::Queen);
        let queen_attackers = self_queen_moves & other_queens;
        if queen_attackers.bits != 0 {
            let queen_attackers = self.extract_squares(queen_attackers);
            attackers.extend(queen_attackers);
        }

        attackers
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

    fn get_legal_king_moves(&self, square: Square, board: &Board) -> Vec<Move> {
        let mut moves = Vec::new();

        let mut other_board = board.clone();
        other_board.swap_active();

        // All moves from the opponent where the king cant move to
        let mut other_moves = Vec::new();

        // Get all sliding moves from the opponent without the king of the current player
        other_board.toggle_other(Piece::King, square);
        let other_sliding_moves = self.get_sliding_moves(&other_board);
        other_moves.extend(other_sliding_moves);
        other_board.toggle_other(Piece::King, square);

        let other_non_sliding_moves = self.get_non_sliding_moves(&other_board);
        other_moves.extend(other_non_sliding_moves);

        let mut forbidden_bb = Bitboard::default();
        for mov in other_moves {
            forbidden_bb |= mov.to;
        }

        let king_moves = self.get_king_moves(board, forbidden_bb);
        moves.extend(king_moves);

        moves
    }

    fn get_king_moves(&self, board: &Board, forbidden: Bitboard) -> Vec<Move> {
        let mut moves = Vec::new();

        let squares = board.get_active_squares_by_piece(Piece::King);
        for from in squares {
            let index = from.index as usize;

            let moves_bb = self.get_single_king_moves(board, index);
            let moves_bb = moves_bb & !forbidden;

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
        // }

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

    fn extract_squares(&self, mut bitboard: Bitboard) -> Vec<Square> {
        let mut moves = Vec::new();

        while bitboard.bits != 0 {
            let index = bitboard.bits.trailing_zeros() as usize;
            let square = Square::index(index);
            bitboard ^= square;

            moves.push(square);
        }

        moves
    }

    fn extract_moves(
        &self,
        piece: Piece,
        board: &Board,
        from: Square,
        moves_bb: Bitboard,
    ) -> Vec<Move> {
        let mut moves = Vec::new();

        let other_occupied = board.get_other_occpuied();
        let squares = self.extract_squares(moves_bb);
        for to in squares {
            let is_attack = other_occupied & to;
            let is_attack = is_attack.bits != 0;

            let mov = Move::new(piece, is_attack, from, to);
            moves.push(mov);
        }

        moves
    }
}
