pub mod error;

use std::{fmt::Display, ops::BitOrAssign, str::FromStr};

use crate::{
    bitboard::{Bitboard, Square},
    board::{Board, Color, ColoredPiece, Piece},
    lookup::{utils::Direction, Lookup},
};

use self::error::{InvalidMoveFormat, MoveError, MoveGeneratorError, PieceNotFound};

#[derive(Debug, PartialEq, Eq)]
pub struct Move {
    pub piece: Piece,
    pub attack: bool,
    pub from: Square,
    pub to: Square,
    pub promoted: bool,
}

impl Move {
    #[rustfmt::skip]
    pub const OO_KING_WHITE: Move = Move::new(Piece::King, false, Square::index(4), Square::index(6), false);
    #[rustfmt::skip]
    pub const OO_ROOK_WHITE: Move = Move::new(Piece::Rook, false, Square::index(7), Square::index(5), false);

    #[rustfmt::skip]
    pub const OOO_KING_WHITE: Move = Move::new(Piece::King, false, Square::index(4), Square::index(2), false);
    #[rustfmt::skip]
    pub const OOO_ROOK_WHITE: Move = Move::new(Piece::Rook, false, Square::index(0), Square::index(3), false);

    #[rustfmt::skip]
    pub const OO_KING_BLACK: Move = Move::new(Piece::King, false, Square::index(60), Square::index(62), false);
    #[rustfmt::skip]
    pub const OO_ROOK_BLACK: Move = Move::new(Piece::Rook, false, Square::index(63), Square::index(61), false);

    #[rustfmt::skip]
    pub const OOO_KING_BLACK: Move = Move::new(Piece::King, false, Square::index(60), Square::index(58), false);
    #[rustfmt::skip]
    pub const OOO_ROOK_BLACK: Move = Move::new(Piece::Rook, false, Square::index(56), Square::index(59), false);

    pub const fn new(piece: Piece, attack: bool, from: Square, to: Square, promoted: bool) -> Self {
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

#[derive(Default, Debug)]
pub struct MoveGenerator;

#[derive(Default, Debug)]
pub struct PinState {
    pins: Bitboard,
    attackers: Bitboard,
}

impl PinState {
    pub fn new(pins: Bitboard, attackers: Bitboard) -> Self {
        Self { pins, attackers }
    }
}

impl BitOrAssign for PinState {
    fn bitor_assign(&mut self, rhs: Self) {
        self.pins |= rhs.pins;
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
        let mut moves = Vec::new();

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
        let mut moves = Vec::new();

        let king = board.get_king_square(board.active)?;
        let attacker_ray = match king {
            Some(king) => match Direction::between(checker, king) {
                Some(direction) => {
                    let mut attacker_ray = Lookup::get_ray(checker, direction);
                    let king_ray = Lookup::get_ray(king, direction);
                    attacker_ray &= !king_ray;
                    attacker_ray
                }
                _ => Bitboard::default(),
            },
            None => Bitboard::default(),
        };

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
        for piece in other_pieces {
            let between = Lookup::get_between(king, piece);
            if between.bits == 0 {
                continue;
            }

            let pinned = between & all_occupied;

            let amount = pinned.bits.count_ones();
            if amount != 1 {
                continue;
            }

            pin_state.pins |= pinned;
            pin_state.attackers |= piece;
        }

        pin_state
    }

    pub fn get_checkers(&self, board: &Board, king: Option<Square>) -> Vec<Square> {
        let king = match king {
            Some(king) => king,
            None => return Vec::new(),
        };

        let mut attackers = Vec::new();

        let forbidden = *board.get_occupied(board.active);

        let self_pawn_moves = self.get_single_pawn_attacks(board, false, king, forbidden);
        let other_pawns = board.get_piece_board(!board.active, Piece::Pawn);
        let pawn_attackers = self_pawn_moves & other_pawns;
        if pawn_attackers.bits != 0 {
            let pawn_attackers = self.extract_squares(pawn_attackers);
            attackers.extend(pawn_attackers);
        }

        let self_knight_moves = self.get_single_knight_moves(king, forbidden);
        let other_knights = board.get_piece_board(!board.active, Piece::Knight);
        let knight_attackers = self_knight_moves & other_knights;
        if knight_attackers.bits != 0 {
            let knight_attackers = self.extract_squares(knight_attackers);
            attackers.extend(knight_attackers);
        }

        let self_bishop_moves = self.get_single_bishop_moves(board, king, forbidden);
        let other_bishops = board.get_piece_board(!board.active, Piece::Bishop);
        let bishop_attackers = self_bishop_moves & other_bishops;
        if bishop_attackers.bits != 0 {
            let bishop_attackers = self.extract_squares(bishop_attackers);
            attackers.extend(bishop_attackers);
        }

        let self_rook_moves = self.get_single_rook_moves(board, king, forbidden);
        let other_rooks = board.get_piece_board(!board.active, Piece::Rook);
        let rook_attackers = self_rook_moves & other_rooks;
        if rook_attackers.bits != 0 {
            let rook_attackers = self.extract_squares(rook_attackers);
            attackers.extend(rook_attackers);
        }

        let self_queen_moves = self_rook_moves | self_bishop_moves;
        let other_queens = board.get_piece_board(!board.active, Piece::Queen);
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
        pin_state: &PinState,
        forbidden: Bitboard,
        attackable: Bitboard,
    ) -> Vec<Move> {
        let mut moves = Vec::new();

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
        let mut moves = Vec::new();

        let bishop_moves = self.get_bishop_moves(board, pin_state, forbidden, attackable);
        moves.extend(bishop_moves);

        let rook_moves = self.get_rook_moves(board, pin_state, forbidden, attackable);
        moves.extend(rook_moves);

        let queen_moves = self.get_queen_moves(board, pin_state, forbidden, attackable);
        moves.extend(queen_moves);

        moves
    }

    fn get_legal_king_moves(&self, board: &Board) -> Result<Vec<Move>, MoveGeneratorError> {
        let pin_state = PinState::default();
        let mut moves = Vec::new();

        let king = match board.get_king_square(board.active)? {
            Some(king) => king,
            None => return Ok(moves),
        };

        let mut other_board = board.clone();
        other_board.swap_active();

        // All moves from the opponent where the king cant move to
        let mut other_moves = Vec::new();

        {
            // Get all sliding moves from the opponent without the king of the current player
            other_board.toggle(!other_board.active, Piece::King, king);

            // Attack every piece even own ones
            let attackable = *other_board.get_all_occupied();
            // Empty because we want to attack every possible pieces (even own ones, because the king
            // could check himself by capturing the piece)
            let forbidden = Bitboard::default();

            let other_sliding_moves =
                self.get_sliding_moves(&other_board, &pin_state, forbidden, attackable);
            other_moves.extend(other_sliding_moves);

            // Toggle it on again
            other_board.toggle(!other_board.active, Piece::King, king);

            let other_non_sliding_moves = self.get_non_sliding_moves(
                &other_board,
                true,
                true,
                &pin_state,
                forbidden,
                attackable,
            );
            other_moves.extend(other_non_sliding_moves);
        }

        let mut forbidden = Bitboard::default();
        for mov in other_moves {
            forbidden |= mov.to;
        }

        forbidden |= *board.get_occupied(board.active);

        let attackable = *board.get_occupied(!board.active);
        let king_moves = self.get_king_moves(board, &pin_state, forbidden, attackable);
        moves.extend(king_moves);

        Ok(moves)
    }

    fn get_king_moves(
        &self,
        board: &Board,
        pin_state: &PinState,
        forbidden: Bitboard,
        attackable: Bitboard,
    ) -> Vec<Move> {
        let mut moves = Vec::new();

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
        let mut moves = Vec::new();

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
        let mut moves = Vec::new();

        let squares = board.get_squares_by_piece(board.active, Piece::Pawn);
        for from in squares {
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
        include_pawn_attacks: bool,
        from: Square,
        forbidden: Bitboard,
    ) -> Bitboard {
        let other_occupied = board.get_occupied(!board.active);

        let attack_mask = Lookup::get_pawn_attacks(board.active, from);
        let mut moves = if include_pawn_attacks {
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
        let mut moves = Vec::new();

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
        let mut moves = Vec::new();

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
        let mut moves = Vec::new();

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
        pin_state: &PinState,
        moves_bb: Bitboard,
        attackable: Bitboard,
    ) -> Vec<Move> {
        let mut moves = Vec::new();

        let is_pinned = pin_state.pins.is_set(from);

        let squares = self.extract_squares(moves_bb);
        for to in squares {
            let is_attack = attackable.is_set(to);
            if is_pinned {
                if !is_attack {
                    continue;
                }

                let removes_attacker = pin_state.attackers.is_set(to);
                if !removes_attacker {
                    continue;
                }
            }

            let mov = Move::new(piece, is_attack, from, to, false);
            moves.push(mov);
        }

        moves
    }
}
