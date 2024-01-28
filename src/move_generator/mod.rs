pub mod error;
mod tests;

use std::{fmt::Display, ops::BitOrAssign, str::FromStr};

use crate::{
    bitboard::{Bitboard, Square},
    board::{Board, Color, ColoredPiece, Piece, EnPassant},
    lookup::{utils::Direction, Lookup},
};

use self::error::{InvalidMoveFormat, MoveError, MoveGeneratorError, PieceNotFound};

#[derive(Debug, PartialEq, Eq)]
pub struct Move {
    pub piece: Piece,
    pub from: Square,
    pub to: Square,
    pub promoted: bool,
    pub attack: bool,
    pub en_passant_capture: Option<Square>,
}

impl Move {
    #[rustfmt::skip]
    pub const OO_KING_WHITE: Move = Move::new(Piece::King, false, Square::index(4), Square::index(6), false, None);
    #[rustfmt::skip]
    pub const OO_ROOK_WHITE: Move = Move::new(Piece::Rook, false, Square::index(7), Square::index(5), false, None);
    #[rustfmt::skip]
    pub const OO_ROOK_REVERSE_WHITE: Move = Move::new(Piece::Rook, false, Square::index(5), Square::index(7), false, None);
    
    #[rustfmt::skip]
    pub const OOO_KING_WHITE: Move = Move::new(Piece::King, false, Square::index(4), Square::index(2), false, None);
    #[rustfmt::skip]
    pub const OOO_ROOK_WHITE: Move = Move::new(Piece::Rook, false, Square::index(0), Square::index(3), false, None);
    #[rustfmt::skip]
    pub const OOO_ROOK_REVERSE_WHITE: Move = Move::new(Piece::Rook, false, Square::index(3), Square::index(0), false, None);

    #[rustfmt::skip]
    pub const OO_KING_BLACK: Move = Move::new(Piece::King, false, Square::index(60), Square::index(62), false, None);
    #[rustfmt::skip]
    pub const OO_ROOK_BLACK: Move = Move::new(Piece::Rook, false, Square::index(63), Square::index(61), false, None);
    #[rustfmt::skip]
    pub const OO_ROOK_REVERSE_BLACK: Move = Move::new(Piece::Rook, false, Square::index(61), Square::index(63), false, None);

    #[rustfmt::skip]
    pub const OOO_KING_BLACK: Move = Move::new(Piece::King, false, Square::index(60), Square::index(58), false, None);
    #[rustfmt::skip]
    pub const OOO_ROOK_BLACK: Move = Move::new(Piece::Rook, false, Square::index(56), Square::index(59), false, None);
    #[rustfmt::skip]
    pub const OOO_ROOK_REVERSE_BLACK: Move = Move::new(Piece::Rook, false, Square::index(59), Square::index(56), false, None);

    pub const fn new(piece: Piece, attack: bool, from: Square, to: Square, promoted: bool, en_passant_capture: Option<Square>) -> Self {
        Self {
            piece,
            attack,
            from,
            to,
            promoted,
            en_passant_capture
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

        // TODO: Add if its an en passant move or not and what piece to capture
        Ok(Self::new(piece, attacked.is_some(), from, to, promoted, None))
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

static mut MOVES_ALL: usize = 0;
static mut MOVES_COUNT: usize = 0;

#[derive(Default, Debug)]
pub struct MoveGenerator;

#[derive(Default, Debug)]
pub struct PinState {
    pins: Bitboard,
    allowed: Bitboard,
    attackers: Bitboard,
}

impl PinState {
    pub fn new(pins: Bitboard, allowed: Bitboard, attackers: Bitboard) -> Self {
        Self { pins, allowed, attackers }
    }
}

impl BitOrAssign for PinState {
    fn bitor_assign(&mut self, rhs: Self) {
        self.pins |= rhs.pins;
        self.allowed |= rhs.allowed;
        self.attackers |= rhs.attackers;
    }
}

impl MoveGenerator {
    pub fn get_legal_moves(&self, board: &Board) -> Result<Vec<Move>, MoveGeneratorError> {
        let king = board.get_king_square(board.active)?;
        let checkers = self.get_checkers(board, king);

        match checkers.len() {
            0 => self.get_unchecked_moves(board),
            1 => self.get_single_check_moves(checkers[0], board),
            2 => self.get_legal_king_moves(board),
            _ => panic!("Invalid amount of checkers"),
        }
    }

    pub fn get_unchecked_moves(&self, board: &Board) -> Result<Vec<Move>, MoveGeneratorError> {
        // TODO: This capacity might change but is here to make it more efficient. 
        let mut moves = Vec::with_capacity(32);

        let king = board.get_king_square(board.active)?;
        let pin_state = self.get_pin_state(board, king);

        let attackable = *board.get_occupied(!board.active);
        let forbidden = *board.get_occupied(board.active);

        let pawn_moves =
            self.get_pawn_moves(board, false, false, &pin_state, forbidden, attackable);
        moves.extend(pawn_moves);

        let knight_moves = self.get_knight_moves(board, &pin_state, forbidden, attackable);
        moves.extend(knight_moves);

        let rook_moves = self.get_rook_moves(board, &pin_state, forbidden, attackable);
        moves.extend(rook_moves);

        let bishop_moves = self.get_bishop_moves(board, &pin_state, forbidden, attackable);
        moves.extend(bishop_moves);

        let queen_moves = self.get_queen_moves(board, &pin_state, forbidden, attackable);
        moves.extend(queen_moves);

        let king_moves = self.get_legal_king_moves(board)?;
        moves.extend(king_moves);

        Ok(moves)
    }

    pub fn get_single_check_moves(
        &self,
        checker: Square,
        board: &Board,
    ) -> Result<Vec<Move>, MoveGeneratorError> {
        // TODO: This capacity might change but is here to make it more efficient. 
        let mut moves = Vec::with_capacity(8);
        
        let king = match board.get_king_square(board.active)? {
            Some(king) => king,
            None => return Ok(moves),
        };

        let attacker_ray = Lookup::get_between(checker, king);

        let unfiltered_moves = self.get_unchecked_moves(board)?;
        for mov in unfiltered_moves {
            let is_blocking = attacker_ray.is_set(mov.to);
            if mov.attack && mov.to == checker {
                // Capture the checking piece
                moves.push(mov);
            } else if mov.piece == Piece::King {
                // Move the king out of check
                moves.push(mov);
            } else if is_blocking {
                // Move a piece between the checking piece and the king
                moves.push(mov);
            }
        }

        Ok(moves)
    }

    pub fn get_pin_state(&self, board: &Board, king: Option<Square>) -> PinState {
        let king = match king {
            Some(king) => king,
            None => return PinState::default(),
        };

        let mut pin_state = self.get_pin_state_by_piece(board, Piece::Queen, king);
        pin_state |= self.get_pin_state_by_piece(board, Piece::Rook, king);
        pin_state |= self.get_pin_state_by_piece(board, Piece::Bishop, king);

        pin_state
    }

    pub fn get_pin_state_by_piece(&self, board: &Board, piece: Piece, king: Square) -> PinState {
        let mut pin_state = PinState::default();

        let all_occupied = board.get_all_occupied();

        let other_pieces = board.get_squares_by_piece(!board.active, piece);
        for piece_sq in other_pieces {
            let direction = Lookup::get_direction(king, piece_sq);
            match (piece, direction) {
                (Piece::Rook, Some(direction)) if direction.is_diagonal() => continue,
                (Piece::Bishop, Some(direction)) if direction.is_straight() => continue,
                (_, Some(direction)) => direction,
                _ => continue,
            };

            let between = Lookup::get_between(king, piece_sq);
            let pinned = between & all_occupied;

            let amount = pinned.bits.count_ones();
            if amount != 1 {
                continue;
            }

            pin_state.pins |= pinned;
            pin_state.allowed |= between;
            pin_state.attackers |= piece_sq;
        }

        pin_state
    }

    pub fn get_checkers(&self, board: &Board, king: Option<Square>) -> Vec<Square> {
        let king = match king {
            Some(king) => king,
            None => return Vec::new(),
        };

        let mut checkers = Vec::new();

        let forbidden = *board.get_occupied(board.active);

        let pawn_attacks = self.get_single_pawn_attacks(board, false, king, forbidden);
        let other_pawns = board.get_piece_board(!board.active, Piece::Pawn);
        let mut attacks = pawn_attacks & other_pawns;

        let knight_attacks = self.get_single_knight_moves(king, forbidden);
        let other_knights = board.get_piece_board(!board.active, Piece::Knight);
        attacks |= knight_attacks & other_knights;

        let bishop_attacks = self.get_single_bishop_moves(board, king, forbidden);
        let other_bishops = board.get_piece_board(!board.active, Piece::Bishop);
        attacks |= bishop_attacks & other_bishops;

        let rook_attacks = self.get_single_rook_moves(board, king, forbidden);
        let other_rooks = board.get_piece_board(!board.active, Piece::Rook);
        attacks |= rook_attacks & other_rooks;

        let queen_attacks = rook_attacks | bishop_attacks;
        let other_queens = board.get_piece_board(!board.active, Piece::Queen);
        attacks |= queen_attacks & other_queens;

        if attacks.bits != 0 {
            let attackers = self.extract_squares(attacks);
            checkers.extend(attackers);
        }

        checkers
    }

    pub fn get_non_sliding_moves(
        &self,
        board: &Board,
        exclude_pawn_moves: bool,
        include_pawn_attacks: bool,
        pin_state: &PinState,
        forbidden: Bitboard,
        attackable: Bitboard,
    ) -> Vec<Move> {
        // TODO: This capacity might change but is here to make it more efficient. 
        let mut moves = Vec::with_capacity(32);

        let pawn_moves = self.get_pawn_moves(
            board,
            exclude_pawn_moves,
            include_pawn_attacks,
            pin_state,
            forbidden,
            attackable,
        );
        moves.extend(pawn_moves);

        let king_moves = self.get_king_moves(board, pin_state, forbidden, attackable);
        moves.extend(king_moves);

        let knight_moves = self.get_knight_moves(board, pin_state, forbidden, attackable);
        moves.extend(knight_moves);

        moves
    }

    pub fn get_sliding_moves(
        &self,
        board: &Board,
        pin_state: &PinState,
        forbidden: Bitboard,
        attackable: Bitboard,
    ) -> Vec<Move> {
        // TODO: This capacity might change but is here to make it more efficient. 
        let mut moves = Vec::with_capacity(32);

        let bishop_moves = self.get_bishop_moves(board, pin_state, forbidden, attackable);
        moves.extend(bishop_moves);

        let rook_moves = self.get_rook_moves(board, pin_state, forbidden, attackable);
        moves.extend(rook_moves);

        let queen_moves = self.get_queen_moves(board, pin_state, forbidden, attackable);
        moves.extend(queen_moves);

        moves
    }

    fn get_legal_king_moves(&self, board: &Board) -> Result<Vec<Move>, MoveGeneratorError> {
        // TODO: This capacity might change but is here to make it more efficient. 
        let mut moves = Vec::with_capacity(8);

        let king = match board.get_king_square(board.active)? {
            Some(king) => king,
            None => return Ok(moves),
        };

        let pin_state = PinState::default();
        
        let mut forbidden = self.get_forbidden_king_squares(board, &pin_state, king);
        forbidden |= *board.get_occupied(board.active);

        let attackable = *board.get_occupied(!board.active);
        let king_moves = self.get_king_moves(board, &pin_state, forbidden, attackable);
        moves.extend(king_moves);

        Ok(moves)
    }

    fn get_forbidden_king_squares(&self, board: &Board, pin_state: &PinState, king: Square) -> Bitboard {
        let mut forbidden = Bitboard::default();

        // Get a copy of the board but viewed from the other player
        let mut board = board.clone();
        board.swap_active();

        // Remove the king so sliding attacks ray through him.
        // Used to forbid squares that will put him still in check.
        board.toggle(!board.active, Piece::King, king);

        // Attack every piece, even own ones
        let attackable = *board.get_all_occupied();


        let sliding = self.get_sliding_moves(&board, &pin_state, Bitboard::default(), attackable);
        for mov in sliding {
            forbidden |= mov.to;
        }

        let non_sliding = self.get_non_sliding_moves(
            &board,
            true,
            true,
            &pin_state,
            Bitboard::default(),
            attackable,
        );
        for mov in non_sliding {
            forbidden |= mov.to;
        }

        forbidden
    }

    fn get_king_moves(
        &self,
        board: &Board,
        pin_state: &PinState,
        forbidden: Bitboard,
        attackable: Bitboard,
    ) -> Vec<Move> {
        // TODO: This capacity might change but is here to make it more efficient. 
        let mut moves = Vec::with_capacity(8);

        let squares = board.get_squares_by_piece(board.active, Piece::King);
        for from in squares {
            let moves_bb = self.get_single_king_moves(from, forbidden);
            let extracted = self.extract_moves(Piece::King, from, pin_state, moves_bb, attackable);
            moves.extend(extracted);
        }

        moves
    }

    fn get_single_king_moves(&self, from: Square, forbidden: Bitboard) -> Bitboard {
        let mut moves = Lookup::get_king_moves(from);
        moves ^= forbidden & moves;
        moves
    }

    fn get_knight_moves(
        &self,
        board: &Board,
        pin_state: &PinState,
        forbidden: Bitboard,
        attackable: Bitboard,
    ) -> Vec<Move> {
        // TODO: This capacity might change but is here to make it more efficient. 
        let mut moves = Vec::with_capacity(8);

        let squares = board.get_squares_by_piece(board.active, Piece::Knight);
        for from in squares {
            let moves_bb = self.get_single_knight_moves(from, forbidden);
            let extracted =
                self.extract_moves(Piece::Knight, from, pin_state, moves_bb, attackable);
            moves.extend(extracted);
        }

        moves
    }

    fn get_single_knight_moves(&self, from: Square, forbidden: Bitboard) -> Bitboard {
        let mut moves = Lookup::get_knight_moves(from);
        moves ^= forbidden & moves;
        moves
    }

    fn get_pawn_moves(
        &self,
        board: &Board,
        exclude_pawn_moves: bool,
        include_pawn_attacks: bool,
        pin_state: &PinState,
        forbidden: Bitboard,
        attackable: Bitboard,
    ) -> Vec<Move> {
        // TODO: This capacity might change but is here to make it more efficient. 
        let mut moves = Vec::with_capacity(8);

        let squares = board.get_squares_by_piece(board.active, Piece::Pawn);
        for from in squares {
            if let Some(en_passant) = board.en_passant { 
                let en_passant_move = self.get_en_passant_move(board, from, en_passant);
                if let Some(en_passant_move) = en_passant_move {
                    moves.push(en_passant_move);
                }
            }

            if !exclude_pawn_moves {
                let moves_bb = self.get_single_pawn_moves(board, from, forbidden);
                let extracted =
                    self.extract_moves(Piece::Pawn, from, pin_state, moves_bb, attackable);
                moves.extend(extracted);
            }

            let moves_bb =
                self.get_single_pawn_attacks(board, include_pawn_attacks, from, forbidden);
            let extracted = self.extract_moves(Piece::Pawn, from, pin_state, moves_bb, attackable);
            moves.extend(extracted);
        }

        moves
    }

    fn get_en_passant_move(&self, board: &Board, from: Square, en_passant: EnPassant) -> Option<Move> {
        let attack_mask = Lookup::get_pawn_attacks(board.active, from);
        let can_en_passant = attack_mask.is_set(en_passant.to_move);
        if can_en_passant {
            let mov = Move::new(Piece::Pawn, false, from, en_passant.to_move, false, Some(en_passant.to_capture));
            return Some(mov);
        }

        None
    }

    fn get_single_pawn_moves(&self, board: &Board, from: Square, forbidden: Bitboard) -> Bitboard {
        let all_occupied = board.get_all_occupied();
        let from_bb: Bitboard = from.into();

        let push_mask = Lookup::get_pawn_pushes(board.active, from);

        let attacking = all_occupied & push_mask;
        if attacking.bits != 0 {
            return Bitboard::default();
        }

        let mut moves = push_mask;
        moves ^= forbidden & moves;

        let double_push_allowed = (Bitboard::RANK_2 & from_bb) | (Bitboard::RANK_7 & from_bb);
        if double_push_allowed.bits == 0 {
            return moves;
        }

        let index = push_mask.get_trailing_index();
        let push_mask = Lookup::get_pawn_pushes(board.active, index);

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
        include_unlegal_attacks: bool,
        from: Square,
        forbidden: Bitboard,
    ) -> Bitboard {
        let other_occupied = board.get_occupied(!board.active);

        let attack_mask = Lookup::get_pawn_attacks(board.active, from);
        let mut moves = if include_unlegal_attacks {
            attack_mask
        } else {
            other_occupied & attack_mask
        };

        moves ^= forbidden & moves;
        moves
    }

    fn get_bishop_moves(
        &self,
        board: &Board,
        pin_state: &PinState,
        forbidden: Bitboard,
        attackable: Bitboard,
    ) -> Vec<Move> {
        // TODO: This capacity might change but is here to make it more efficient. 
        let mut moves = Vec::with_capacity(8);

        let squares = board.get_squares_by_piece(board.active, Piece::Bishop);
        for from in squares {
            let moves_bb = self.get_single_bishop_moves(board, from, forbidden);
            let extracted =
                self.extract_moves(Piece::Bishop, from, pin_state, moves_bb, attackable);
            moves.extend(extracted);
        }

        moves
    }

    fn get_single_bishop_moves(
        &self,
        board: &Board,
        from: Square,
        forbidden: Bitboard,
    ) -> Bitboard {
        let all_occupied = *board.get_all_occupied();
        let mut moves = Bitboard::default();

        moves |= self.get_ray_moves(from, all_occupied, Direction::NorthEast, false);
        moves |= self.get_ray_moves(from, all_occupied, Direction::SouthEast, true);
        moves |= self.get_ray_moves(from, all_occupied, Direction::SouthWest, true);
        moves |= self.get_ray_moves(from, all_occupied, Direction::NorthWest, false);

        moves ^= forbidden & moves;
        moves
    }

    fn get_rook_moves(
        &self,
        board: &Board,
        pin_state: &PinState,
        forbidden: Bitboard,
        attackable: Bitboard,
    ) -> Vec<Move> {
        // TODO: This capacity might change but is here to make it more efficient. 
        let mut moves = Vec::with_capacity(8);

        let squares = board.get_squares_by_piece(board.active, Piece::Rook);
        for from in squares {
            let moves_bb = self.get_single_rook_moves(board, from, forbidden);
            let extracted = self.extract_moves(Piece::Rook, from, pin_state, moves_bb, attackable);
            moves.extend(extracted);
        }

        moves
    }

    fn get_single_rook_moves(&self, board: &Board, from: Square, forbidden: Bitboard) -> Bitboard {
        let all_occupied = *board.get_all_occupied();
        let mut moves = Bitboard::default();

        moves |= self.get_ray_moves(from, all_occupied, Direction::North, false);
        moves |= self.get_ray_moves(from, all_occupied, Direction::East, false);
        moves |= self.get_ray_moves(from, all_occupied, Direction::South, true);
        moves |= self.get_ray_moves(from, all_occupied, Direction::West, true);

        moves ^= forbidden & moves;
        moves
    }

    fn get_ray_moves(
        &self,
        from: Square,
        forbidden: Bitboard,
        direction: Direction,
        leading: bool,
    ) -> Bitboard {
        let mut moves = Bitboard::default();

        let ray = Lookup::get_ray(from, direction);
        moves |= ray;

        let blocking = ray & forbidden;
        if blocking.bits != 0 {
            let blocker_index = match leading {
                false => blocking.get_trailing_index(),
                true => blocking.get_leading_index(),
            };

            moves &= !Lookup::get_ray(blocker_index, direction);
        }

        moves
    }

    fn get_queen_moves(
        &self,
        board: &Board,
        pin_state: &PinState,
        forbidden: Bitboard,
        attackable: Bitboard,
    ) -> Vec<Move> {
        // TODO: This capacity might change but is here to make it more efficient. 
        let mut moves = Vec::with_capacity(8);

        let squares = board.get_squares_by_piece(board.active, Piece::Queen);
        for from in squares {
            let bishop_bb = self.get_single_bishop_moves(board, from, forbidden);
            let rook_bb = self.get_single_rook_moves(board, from, forbidden);
            let moves_bb = bishop_bb | rook_bb;

            let extracted = self.extract_moves(Piece::Queen, from, pin_state, moves_bb, attackable);
            moves.extend(extracted);
        }

        moves
    }

    fn extract_squares(&self, mut bitboard: Bitboard) -> Vec<Square> {
        // TODO: This capacity might change but is here to make it more efficient. 
        let mut moves = Vec::with_capacity(8);

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
        pin_state: &PinState,
        moves_bb: Bitboard,
        attackable: Bitboard,
    ) -> Vec<Move> {
        // TODO: This capacity might change but is here to make it more efficient. 
        let mut moves = Vec::with_capacity(8);

        let is_pinned = pin_state.pins.is_set(from);

        let squares = self.extract_squares(moves_bb);
        for to in squares {
            let is_attack = attackable.is_set(to);
            if is_pinned {
                // TODO: This is sometimes not legal to do 
                let is_allowed = pin_state.allowed.is_set(to);
                if !is_allowed && !is_attack {
                    continue;
                }

                let removes_attacker = pin_state.attackers.is_set(to);
                if is_attack && !removes_attacker {
                    continue;
                }
            }

            let mov = Move::new(piece, is_attack, from, to, false, None);
            moves.push(mov);
        }

        moves
    }
}
