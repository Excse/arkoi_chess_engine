pub mod error;

use std::{collections::HashMap, fmt::Display, str::FromStr};

use crate::{
    bitboard::{Bitboard, Square},
    board::{Board, Color, ColoredPiece, Piece},
    lookup::{
        tables::{BETWEEN_LOOKUP, KING_MOVES, KNIGHT_MOVES, PAWN_ATTACKS, PAWN_PUSHES},
        utils::Direction,
    },
};

use self::error::{InvalidMoveFormat, MoveError, PieceNotFound};

#[derive(Debug)]
pub struct Move {
    pub piece: Piece,
    pub attack: bool,
    pub from: Square,
    pub to: Square,
    pub promoted: bool,
}

impl Move {
    pub fn new(piece: Piece, attack: bool, from: Square, to: Square, promoted: bool) -> Self {
        Self {
            piece,
            attack,
            from,
            to,
            promoted,
        }
    }

    pub fn parse(input: String, board: &Board) -> Result<Self, MoveError> {
        let promoted = input.len() == 5;

        if input.len() != 4 && !promoted {
            return Err(InvalidMoveFormat::new(input.clone()).into());
        }

        let from = Square::from_str(&input[0..2])?;
        let to = Square::from_str(&input[2..4])?;

        let piece = board
            .get_piece_type(board.active, from)
            .ok_or(PieceNotFound::new(from.to_string()))?;
        let attacked = board.get_piece_type(!board.active, to);

        let piece = match input.len() {
            5 => {
                let piece = input
                    .chars()
                    .nth(4)
                    .ok_or(InvalidMoveFormat::new(input.clone()))?;
                let colored_piece = ColoredPiece::from_fen(piece)?;
                colored_piece.piece
            }
            _ => piece,
        };

        Ok(Self::new(piece, attacked.is_some(), from, to, promoted))
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let from = self.from.to_string();
        let to = self.to.to_string();

        // TODO: Check if promotion is always in lowercase
        match self.promoted {
            true => {
                let colored_piece = ColoredPiece::new(self.piece, Color::Black);
                write!(f, "{}{}{}", from, to, colored_piece.to_fen())
            }
            false => write!(f, "{}{}", from, to),
        }
    }
}

#[derive(Default)]
pub struct MoveGenerator;

impl MoveGenerator {
    pub fn get_legal_moves(&self, board: &Board) -> Vec<Move> {
        let king = board.get_own_king_square();
        let checkers = self.get_checkers(board, king);

        match checkers.len() {
            0 => self.get_unchecked_moves(board),
            1 => self.get_single_check_moves(checkers[0], board),
            2 => self.get_double_check_moves(board),
            _ => panic!("Invalid amount of checkers"),
        }
    }

    // TODO: Handle unwrap
    pub fn get_single_check_moves(&self, checker: Square, board: &Board) -> Vec<Move> {
        let mut moves = Vec::new();

        let king = board.get_own_king_square();

        let attacker_ray = match Direction::between(checker, king) {
            Some(attacker_direction) => {
                let mut attacker_ray = attacker_direction.ray(checker.index);
                let king_ray = attacker_direction.ray(king.index);
                attacker_ray &= !king_ray;
                attacker_ray
            }
            _ => 0,
        };

        let unfiltered_moves = self.get_unchecked_moves(board);
        for mov in unfiltered_moves {
            let is_blocking = mov.to & attacker_ray;
            if mov.attack && mov.to == checker {
                // Capture the checking piece
                moves.push(mov);
            } else if mov.piece == Piece::King {
                // Move the king out of check
                moves.push(mov);
            } else if is_blocking.bits != 0 {
                // Move a piece between the checking piece and the king
                moves.push(mov);
            }
        }

        moves
    }

    pub fn get_double_check_moves(&self, board: &Board) -> Vec<Move> {
        self.get_legal_king_moves(board)
    }

    // TODO: Clean up the code & fix bugs, this is a mess
    pub fn get_unchecked_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::new();

        let king = board.get_own_king_square();
        let pinned = self.get_pinned_pieces(board, king);

        let forbidden = *board.get_own_occupied();
        let attackable = *board.get_other_occupied();

        let pawn_moves = self.get_pawn_moves(board, false, false, forbidden, attackable);
        for mov in pawn_moves {
            let is_pinned = pinned.get(&mov.from);
            let should_add = match is_pinned {
                Some(&square) => mov.to == square,
                None => true,
            };

            if should_add {
                moves.push(mov);
            }
        }

        let knight_moves = self.get_knight_moves(board, forbidden, attackable);
        for mov in knight_moves {
            let is_pinned = pinned.get(&mov.from);
            let should_add = match is_pinned {
                Some(&square) => mov.to == square,
                None => true,
            };

            if should_add {
                moves.push(mov);
            }
        }

        let rook_moves = self.get_rook_moves(board, forbidden, attackable);
        for mov in rook_moves {
            let is_pinned = pinned.get(&mov.from);
            let should_add = match is_pinned {
                Some(&square) => mov.to == square,
                None => true,
            };

            if should_add {
                moves.push(mov);
            }
        }

        let bishop_moves = self.get_bishop_moves(board, forbidden, attackable);
        for mov in bishop_moves {
            let is_pinned = pinned.get(&mov.from);
            let should_add = match is_pinned {
                Some(&square) => mov.to == square,
                None => true,
            };

            if should_add {
                moves.push(mov);
            }
        }

        let queen_moves = self.get_queen_moves(board, forbidden, attackable);
        for mov in queen_moves {
            let is_pinned = pinned.get(&mov.from);
            let should_add = match is_pinned {
                Some(&square) => mov.to == square,
                None => true,
            };

            if should_add {
                moves.push(mov);
            }
        }

        let king_moves = self.get_legal_king_moves(board);
        moves.extend(king_moves);

        moves
    }

    pub fn get_pinned_pieces(&self, board: &Board, king: Square) -> HashMap<Square, Square> {
        let mut attackers = HashMap::new();

        let all_occupied = board.get_all_occupied();

        let other_queens = board.get_other_squares_by_piece(Piece::Queen);
        for queen in other_queens {
            let between = Bitboard::bits(BETWEEN_LOOKUP[king.index][queen.index]);
            if between.bits == 0 {
                continue;
            }

            let pinned = between & all_occupied;

            let amount = pinned.bits.count_ones();
            if amount != 1 {
                continue;
            }

            let squares = self.extract_squares(pinned);
            for square in squares {
                attackers.insert(square, queen);
            }
        }

        let other_rooks = board.get_other_squares_by_piece(Piece::Rook);
        for rook in other_rooks {
            let between = Bitboard::bits(BETWEEN_LOOKUP[king.index][rook.index]);
            if between.bits == 0 {
                continue;
            }

            let pinned = between & all_occupied;

            let amount = pinned.bits.count_ones();
            if amount != 1 {
                continue;
            }

            let squares = self.extract_squares(pinned);
            for square in squares {
                attackers.insert(square, rook);
            }
        }

        let other_bishops = board.get_other_squares_by_piece(Piece::Bishop);
        for bishop in other_bishops {
            let between = Bitboard::bits(BETWEEN_LOOKUP[king.index][bishop.index]);
            if between.bits == 0 {
                continue;
            }

            let pinned = between & all_occupied;

            let amount = pinned.bits.count_ones();
            if amount != 1 {
                continue;
            }

            let squares = self.extract_squares(pinned);
            for square in squares {
                attackers.insert(square, bishop);
            }
        }

        attackers
    }

    pub fn get_checkers(&self, board: &Board, king: Square) -> Vec<Square> {
        let mut attackers = Vec::new();

        let forbidden = *board.get_own_occupied();
        let index = king.index as usize;

        let self_pawn_moves = self.get_single_pawn_attacks(board, false, index, forbidden);
        let other_pawns = board.get_other_piece_board(Piece::Pawn);
        let pawn_attackers = self_pawn_moves & other_pawns;
        if pawn_attackers.bits != 0 {
            let pawn_attackers = self.extract_squares(pawn_attackers);
            attackers.extend(pawn_attackers);
        }

        let self_knight_moves = self.get_single_knight_moves(index, forbidden);
        let other_knights = board.get_other_piece_board(Piece::Knight);
        let knight_attackers = self_knight_moves & other_knights;
        if knight_attackers.bits != 0 {
            let knight_attackers = self.extract_squares(knight_attackers);
            attackers.extend(knight_attackers);
        }

        let self_bishop_moves = self.get_single_bishop_moves(board, index, forbidden);
        let other_bishops = board.get_other_piece_board(Piece::Bishop);
        let bishop_attackers = self_bishop_moves & other_bishops;
        if bishop_attackers.bits != 0 {
            let bishop_attackers = self.extract_squares(bishop_attackers);
            attackers.extend(bishop_attackers);
        }

        let self_rook_moves = self.get_single_rook_moves(board, index, forbidden);
        let other_rooks = board.get_other_piece_board(Piece::Rook);
        let rook_attackers = self_rook_moves & other_rooks;
        if rook_attackers.bits != 0 {
            let rook_attackers = self.extract_squares(rook_attackers);
            attackers.extend(rook_attackers);
        }

        let self_queen_moves = self_rook_moves | self_bishop_moves;
        let other_queens = board.get_other_piece_board(Piece::Queen);
        let queen_attackers = self_queen_moves & other_queens;
        if queen_attackers.bits != 0 {
            let queen_attackers = self.extract_squares(queen_attackers);
            attackers.extend(queen_attackers);
        }

        attackers
    }

    pub fn get_non_sliding_moves(
        &self,
        board: &Board,
        exclude_pawn_moves: bool,
        include_pawn_attacks: bool,
        forbidden: Bitboard,
        attackable: Bitboard,
    ) -> Vec<Move> {
        let mut moves = Vec::new();

        let pawn_moves = self.get_pawn_moves(
            board,
            exclude_pawn_moves,
            include_pawn_attacks,
            forbidden,
            attackable,
        );
        moves.extend(pawn_moves);

        let king_moves = self.get_king_moves(board, forbidden, attackable);
        moves.extend(king_moves);

        let knight_moves = self.get_knight_moves(board, forbidden, attackable);
        moves.extend(knight_moves);

        moves
    }

    pub fn get_sliding_moves(
        &self,
        board: &Board,
        forbidden: Bitboard,
        attackable: Bitboard,
    ) -> Vec<Move> {
        let mut moves = Vec::new();

        let bishop_moves = self.get_bishop_moves(board, forbidden, attackable);
        moves.extend(bishop_moves);

        let rook_moves = self.get_rook_moves(board, forbidden, attackable);
        moves.extend(rook_moves);

        let queen_moves = self.get_queen_moves(board, forbidden, attackable);
        moves.extend(queen_moves);

        moves
    }

    fn get_legal_king_moves(&self, board: &Board) -> Vec<Move> {
        let mut moves = Vec::new();

        let king = board.get_own_king_square();

        let mut other_board = board.clone();
        other_board.swap_active();

        // All moves from the opponent where the king cant move to
        let mut other_moves = Vec::new();

        {
            // Get all sliding moves from the opponent without the king of the current player
            other_board.toggle_other(Piece::King, king);

            // Attack every piece even own ones
            let attackable = *other_board.get_all_occupied();
            // Empty because we want to attack every possible pieces (even own ones, because the king
            // could check himself by capturing the piece)
            let forbidden = Bitboard::default();

            let other_sliding_moves = self.get_sliding_moves(&other_board, forbidden, attackable);
            other_moves.extend(other_sliding_moves);

            // Toggle it on again
            other_board.toggle_other(Piece::King, king);

            let other_non_sliding_moves =
                self.get_non_sliding_moves(&other_board, true, true, forbidden, attackable);
            other_moves.extend(other_non_sliding_moves);
        }

        let mut forbidden = Bitboard::default();
        for mov in other_moves {
            forbidden |= mov.to;
        }

        forbidden |= *board.get_own_occupied();

        let attackable = *board.get_other_occupied();
        let king_moves = self.get_king_moves(board, forbidden, attackable);
        moves.extend(king_moves);

        moves
    }

    fn get_king_moves(
        &self,
        board: &Board,
        forbidden: Bitboard,
        attackable: Bitboard,
    ) -> Vec<Move> {
        let mut moves = Vec::new();

        let squares = board.get_active_squares_by_piece(Piece::King);
        for from in squares {
            let index = from.index as usize;

            let moves_bb = self.get_single_king_moves(index, forbidden);
            let extracted = self.extract_moves(Piece::King, from, moves_bb, attackable);
            moves.extend(extracted);
        }

        moves
    }

    fn get_single_king_moves(&self, index: usize, forbidden: Bitboard) -> Bitboard {
        let moves_mask = KING_MOVES[index];

        let mut moves = Bitboard::bits(moves_mask);
        moves ^= forbidden & moves;
        moves
    }

    fn get_knight_moves(
        &self,
        board: &Board,
        forbidden: Bitboard,
        attackable: Bitboard,
    ) -> Vec<Move> {
        let mut moves = Vec::new();

        let squares = board.get_active_squares_by_piece(Piece::Knight);
        for from in squares {
            let index = from.index as usize;

            let moves_bb = self.get_single_knight_moves(index, forbidden);
            let extracted = self.extract_moves(Piece::Knight, from, moves_bb, attackable);
            moves.extend(extracted);
        }

        moves
    }

    fn get_single_knight_moves(&self, index: usize, forbidden: Bitboard) -> Bitboard {
        let moves_mask = KNIGHT_MOVES[index];

        let mut moves = Bitboard::bits(moves_mask);
        moves ^= forbidden & moves;
        moves
    }

    fn get_pawn_moves(
        &self,
        board: &Board,
        exclude_pawn_moves: bool,
        include_pawn_attacks: bool,
        forbidden: Bitboard,
        attackable: Bitboard,
    ) -> Vec<Move> {
        let mut moves = Vec::new();

        let squares = board.get_active_squares_by_piece(Piece::Pawn);
        for from in squares {
            let from_bb: Bitboard = from.into();
            let index = from.index as usize;

            if !exclude_pawn_moves {
                let moves_bb = self.get_single_pawn_moves(board, index, from_bb, forbidden);
                let extracted = self.extract_moves(Piece::Pawn, from, moves_bb, attackable);
                moves.extend(extracted);
            }

            let moves_bb =
                self.get_single_pawn_attacks(board, include_pawn_attacks, index, forbidden);
            let extracted = self.extract_moves(Piece::Pawn, from, moves_bb, attackable);
            moves.extend(extracted);
        }

        moves
    }

    fn get_single_pawn_moves(
        &self,
        board: &Board,
        index: usize,
        from_bb: Bitboard,
        forbidden: Bitboard,
    ) -> Bitboard {
        let active_index = board.active.index();
        let all_occupied = board.get_all_occupied();

        let push_mask = PAWN_PUSHES[active_index][index];
        let attacking = all_occupied & push_mask;
        if attacking.bits != 0 {
            return Bitboard::default();
        }

        let mut moves = Bitboard::bits(push_mask);
        moves ^= forbidden & moves;

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
        moves ^= forbidden & moves;
        moves
    }

    fn get_single_pawn_attacks(
        &self,
        board: &Board,
        include_pawn_attacks: bool,
        index: usize,
        forbidden: Bitboard,
    ) -> Bitboard {
        let other_occupied = board.get_other_occupied();
        let active_index = board.active.index();

        let attack_mask = PAWN_ATTACKS[active_index][index];
        let mut moves = if include_pawn_attacks {
            Bitboard::bits(attack_mask)
        } else {
            other_occupied & attack_mask
        };

        moves ^= forbidden & moves;
        moves
    }

    fn get_bishop_moves(
        &self,
        board: &Board,
        forbidden: Bitboard,
        attackable: Bitboard,
    ) -> Vec<Move> {
        let mut moves = Vec::new();

        let squares = board.get_active_squares_by_piece(Piece::Bishop);
        for from in squares {
            let index = from.index as usize;

            let moves_bb = self.get_single_bishop_moves(board, index, forbidden);
            let extracted = self.extract_moves(Piece::Bishop, from, moves_bb, attackable);
            moves.extend(extracted);
        }

        moves
    }

    fn get_single_bishop_moves(
        &self,
        board: &Board,
        index: usize,
        forbidden: Bitboard,
    ) -> Bitboard {
        let all_occupied = *board.get_all_occupied();
        let mut moves = Bitboard::default();

        moves |= self.get_ray_moves(index, all_occupied, Direction::NorthEast, false);
        moves |= self.get_ray_moves(index, all_occupied, Direction::SouthEast, true);
        moves |= self.get_ray_moves(index, all_occupied, Direction::SouthWest, true);
        moves |= self.get_ray_moves(index, all_occupied, Direction::NorthWest, false);

        moves ^= forbidden & moves;

        moves
    }

    fn get_rook_moves(
        &self,
        board: &Board,
        forbidden: Bitboard,
        attackable: Bitboard,
    ) -> Vec<Move> {
        let mut moves = Vec::new();

        let squares = board.get_active_squares_by_piece(Piece::Rook);
        for from in squares {
            let index = from.index as usize;

            let moves_bb = self.get_single_rook_moves(board, index, forbidden);
            let extracted = self.extract_moves(Piece::Rook, from, moves_bb, attackable);
            moves.extend(extracted);
        }

        moves
    }

    fn get_single_rook_moves(&self, board: &Board, index: usize, forbidden: Bitboard) -> Bitboard {
        let all_occupied = *board.get_all_occupied();
        let mut moves = Bitboard::default();

        moves |= self.get_ray_moves(index, all_occupied, Direction::North, false);
        moves |= self.get_ray_moves(index, all_occupied, Direction::East, false);
        moves |= self.get_ray_moves(index, all_occupied, Direction::South, true);
        moves |= self.get_ray_moves(index, all_occupied, Direction::West, true);

        moves ^= forbidden & moves;
        moves
    }

    fn get_ray_moves(
        &self,
        index: usize,
        forbidden: Bitboard,
        direction: Direction,
        leading: bool,
    ) -> Bitboard {
        let mut moves = Bitboard::default();

        let ray = direction.ray(index);
        moves |= ray;

        let blocking = ray & forbidden;
        if blocking.bits != 0 {
            let blocker_index = match leading {
                false => blocking.get_trailing_index() as usize,
                true => blocking.get_leading_index() as usize,
            };

            moves &= !direction.ray(blocker_index);
        }

        moves
    }

    fn get_queen_moves(
        &self,
        board: &Board,
        forbidden: Bitboard,
        attackable: Bitboard,
    ) -> Vec<Move> {
        let mut moves = Vec::new();

        let squares = board.get_active_squares_by_piece(Piece::Queen);
        for from in squares {
            let index = from.index as usize;

            let bishop_bb = self.get_single_bishop_moves(board, index, forbidden);
            let rook_bb = self.get_single_rook_moves(board, index, forbidden);
            let moves_bb = bishop_bb | rook_bb;

            let extracted = self.extract_moves(Piece::Queen, from, moves_bb, attackable);
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
        from: Square,
        moves_bb: Bitboard,
        attackable: Bitboard,
    ) -> Vec<Move> {
        let mut moves = Vec::new();

        let squares = self.extract_squares(moves_bb);
        for to in squares {
            let is_attack = attackable & to;
            let is_attack = is_attack.bits != 0;

            let mov = Move::new(piece, is_attack, from, to, false);
            moves.push(mov);
        }

        moves
    }
}
