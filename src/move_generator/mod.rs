pub mod error;
pub mod mov;
mod tests;

use crate::{
    bitboard::{constants::*, square::Square, Bitboard},
    board::{color::Color, piece::Piece, Board},
};

use self::{
    error::MoveGeneratorError,
    mov::{
        AttackMove, CastleMove, EnPassant, EnPassantMove, Move, MoveKind, NormalMove, PromotionMove,
    },
};

#[derive(Debug)]
pub struct PinState {
    pins: [Bitboard; Board::SIZE],
    cant_en_passant: bool,
}

impl Default for PinState {
    fn default() -> Self {
        Self {
            pins: [Bitboard::default(); Board::SIZE],
            cant_en_passant: false,
        }
    }
}

pub struct MoveGenerator;

#[derive(Debug)]
pub struct MoveState {
    pub is_checkmate: bool,
    pub is_stalemate: bool,
    pub is_check: bool,
    pub moves: Vec<Move>,
}

impl MoveState {
    pub fn new(is_checkmate: bool, is_stalemate: bool, is_check: bool, moves: Vec<Move>) -> Self {
        Self {
            is_checkmate,
            is_stalemate,
            is_check,
            moves,
        }
    }
}

impl MoveGenerator {
    pub fn get_legal_moves(board: &Board) -> Result<MoveState, MoveGeneratorError> {
        let checkers = Self::get_checkers(board);

        let moves = match checkers.len() {
            0 => Self::get_unchecked_moves(board),
            1 => Self::get_single_check_moves(checkers[0], board),
            2 => Self::get_legal_king_moves(board, true),
            _ => panic!("Invalid amount of checkers"),
        }?;

        let is_checkmate = checkers.len() > 0 && moves.len() == 0;
        let is_stalemate = checkers.len() == 0 && moves.len() == 0;
        let is_check = checkers.len() > 0 && !is_checkmate && !is_stalemate;

        let board_state = MoveState::new(is_checkmate, is_stalemate, is_check, moves);
        Ok(board_state)
    }

    pub fn get_unchecked_moves(board: &Board) -> Result<Vec<Move>, MoveGeneratorError> {
        // TODO: This capacity might change but is here to make it more efficient.
        let mut moves = Vec::with_capacity(32);

        let king = board.get_king_square(board.active);
        let pin_state = Self::get_pin_state(board, king);

        let attackable = *board.get_occupied(!board.active);
        let forbidden = *board.get_occupied(board.active);

        let pawn_moves =
            Self::get_pawn_moves(board, false, false, &pin_state, forbidden, attackable);
        moves.extend(pawn_moves);

        let knight_moves = Self::get_knight_moves(board, &pin_state, forbidden, attackable);
        moves.extend(knight_moves);

        let rook_moves = Self::get_rook_moves(board, &pin_state, forbidden, attackable);
        moves.extend(rook_moves);

        let bishop_moves = Self::get_bishop_moves(board, &pin_state, forbidden, attackable);
        moves.extend(bishop_moves);

        let queen_moves = Self::get_queen_moves(board, &pin_state, forbidden, attackable);
        moves.extend(queen_moves);

        let king_moves = Self::get_legal_king_moves(board, false)?;
        moves.extend(king_moves);

        Ok(moves)
    }

    pub fn get_single_check_moves(
        checker: Square,
        board: &Board,
    ) -> Result<Vec<Move>, MoveGeneratorError> {
        // TODO: This capacity might change but is here to make it more efficient.
        let mut moves = Vec::with_capacity(8);

        let king = board.get_king_square(board.active);
        let attacker_ray = checker.get_between(king);

        let unfiltered_moves = Self::get_unchecked_moves(board)?;
        for mov in unfiltered_moves {
            if matches!(mov.kind, MoveKind::Castle(_)) {
                continue;
            }

            let is_blocking = attacker_ray.is_set(mov.to);
            match mov.get_attacking_square() {
                Some(attacked) if attacked == checker => {
                    // Capture the checking piece
                    moves.push(mov);
                    continue;
                }
                _ => {}
            }

            if mov.piece == Piece::King {
                // Move the king out of check
                moves.push(mov);
            } else if is_blocking {
                // Move a piece between the checking piece and the king
                moves.push(mov);
            }
        }

        Ok(moves)
    }

    pub fn get_pin_state(board: &Board, king: Square) -> PinState {
        let mut pin_state = PinState::default();

        Self::get_pin_state_by_piece(board, &mut pin_state, Piece::Queen, king);
        Self::get_pin_state_by_piece(board, &mut pin_state, Piece::Rook, king);
        Self::get_pin_state_by_piece(board, &mut pin_state, Piece::Bishop, king);

        pin_state
    }

    pub fn get_pin_state_by_piece(
        board: &Board,
        pin_state: &mut PinState,
        piece: Piece,
        king: Square,
    ) {
        let all_occupied = board.get_all_occupied();

        let other_pieces = board.get_squares_by_piece(!board.active, piece);
        for piece_sq in other_pieces {
            let direction = king.get_direction(piece_sq);
            let direction = match (piece, direction) {
                (Piece::Rook, Some(direction)) if direction.is_diagonal() => continue,
                (Piece::Bishop, Some(direction)) if direction.is_straight() => continue,
                (_, Some(direction)) => direction,
                _ => continue,
            };

            let between = king.get_between(piece_sq);
            let pinned = between & all_occupied;

            if direction.is_horizontal() {
                if let Some(en_passant) = board.en_passant {
                    if pinned.is_set(en_passant.to_capture) {
                        let test = pinned ^ en_passant.to_capture;
                        if test.bits.count_ones() == 1 {
                            let pinned = test.get_leading_index();
                            let typ = board.get_piece_type(Square::index(pinned)).unwrap();
                            if typ.piece == Piece::Pawn {
                                pin_state.cant_en_passant = true;
                            }
                        }
                    }
                }
            }

            let amount = pinned.bits.count_ones();
            if amount != 1 {
                continue;
            }

            let pinned = pinned.get_leading_index();

            let allowed = between ^ piece_sq;
            pin_state.pins[pinned] = allowed;
        }
    }

    pub fn get_checkers(board: &Board) -> Vec<Square> {
        let mut checkers = Vec::new();

        let king = board.get_king_square(board.active);
        let forbidden = *board.get_occupied(board.active);
        let all_occupied = *board.get_all_occupied();

        let pawn_attacks = Self::get_single_pawn_attacks(board, false, king, forbidden);
        let other_pawns = board.get_piece_board(!board.active, Piece::Pawn);
        let mut attacks = pawn_attacks & other_pawns;

        let knight_attacks = Self::get_single_knight_moves(king, forbidden);
        let other_knights = board.get_piece_board(!board.active, Piece::Knight);
        attacks |= knight_attacks & other_knights;

        let bishop_attacks = king.get_bishop_attacks(all_occupied) & !forbidden;
        let other_bishops = board.get_piece_board(!board.active, Piece::Bishop);
        attacks |= bishop_attacks & other_bishops;

        let rook_attacks = king.get_rook_attacks(all_occupied) & !forbidden;
        let other_rooks = board.get_piece_board(!board.active, Piece::Rook);
        attacks |= rook_attacks & other_rooks;

        let queen_attacks = rook_attacks | bishop_attacks;
        let other_queens = board.get_piece_board(!board.active, Piece::Queen);
        attacks |= queen_attacks & other_queens;

        if attacks.bits != 0 {
            let attackers = Self::extract_squares(attacks);
            checkers.extend(attackers);
        }

        checkers
    }

    pub fn get_non_sliding_moves(
        board: &Board,
        exclude_pawn_moves: bool,
        include_pawn_attacks: bool,
        pin_state: &PinState,
        forbidden: Bitboard,
        attackable: Bitboard,
    ) -> Result<Vec<Move>, MoveGeneratorError> {
        // TODO: This capacity might change but is here to make it more efficient.
        let mut moves = Vec::with_capacity(32);

        let pawn_moves = Self::get_pawn_moves(
            board,
            exclude_pawn_moves,
            include_pawn_attacks,
            pin_state,
            forbidden,
            attackable,
        );
        moves.extend(pawn_moves);

        let king_moves = Self::get_king_moves(board, pin_state, forbidden, attackable)?;
        moves.extend(king_moves);

        let knight_moves = Self::get_knight_moves(board, pin_state, forbidden, attackable);
        moves.extend(knight_moves);

        Ok(moves)
    }

    pub fn get_sliding_moves(
        board: &Board,
        pin_state: &PinState,
        forbidden: Bitboard,
        attackable: Bitboard,
    ) -> Vec<Move> {
        // TODO: This capacity might change but is here to make it more efficient.
        let mut moves = Vec::with_capacity(32);

        let bishop_moves = Self::get_bishop_moves(board, pin_state, forbidden, attackable);
        moves.extend(bishop_moves);

        let rook_moves = Self::get_rook_moves(board, pin_state, forbidden, attackable);
        moves.extend(rook_moves);

        let queen_moves = Self::get_queen_moves(board, pin_state, forbidden, attackable);
        moves.extend(queen_moves);

        moves
    }

    fn get_legal_king_moves(
        board: &Board,
        exclude_castling: bool,
    ) -> Result<Vec<Move>, MoveGeneratorError> {
        // TODO: This capacity might change but is here to make it more efficient.
        let mut moves = Vec::with_capacity(8);

        let king = board.get_king_square(board.active);
        let pin_state = PinState::default();

        let mut forbidden = Self::get_forbidden_king_squares(board, &pin_state, king)?;
        forbidden |= *board.get_occupied(board.active);

        let attackable = *board.get_occupied(!board.active);
        let king_moves = Self::get_king_moves(board, &pin_state, forbidden, attackable)?;
        moves.extend(king_moves);

        if !exclude_castling {
            let castle_moves = Self::get_king_castle_moves(board, forbidden);
            moves.extend(castle_moves);
        }

        Ok(moves)
    }

    fn get_forbidden_king_squares(
        board: &Board,
        pin_state: &PinState,
        king: Square,
    ) -> Result<Bitboard, MoveGeneratorError> {
        let mut forbidden = Bitboard::default();

        // Get a copy of the board but viewed from the other player
        let mut board = board.clone();
        board.swap_active();

        // Remove the king so sliding attacks ray through him.
        // Used to forbid squares that will put him still in check.
        board.toggle(!board.active, Piece::King, king);

        // Attack every piece, even own ones
        let attackable = *board.get_all_occupied();

        let sliding = Self::get_sliding_moves(&board, &pin_state, Bitboard::default(), attackable);
        for mov in sliding {
            forbidden |= mov.to;
        }

        let non_sliding = Self::get_non_sliding_moves(
            &board,
            true,
            true,
            &pin_state,
            Bitboard::default(),
            attackable,
        )?;
        for mov in non_sliding {
            forbidden |= mov.to;
        }

        Ok(forbidden)
    }

    fn get_king_moves(
        board: &Board,
        pin_state: &PinState,
        forbidden: Bitboard,
        attackable: Bitboard,
    ) -> Result<Vec<Move>, MoveGeneratorError> {
        // TODO: This capacity might change but is here to make it more efficient.
        let mut moves = Vec::with_capacity(8);

        let king = board.get_king_square(board.active);

        let moves_bb = Self::get_single_king_moves(king, forbidden);
        let extracted = Self::extract_moves(Piece::King, king, pin_state, moves_bb, attackable);
        moves.extend(extracted);

        Ok(moves)
    }

    // TODO: make this better
    fn get_king_castle_moves(board: &Board, forbidden: Bitboard) -> Vec<Move> {
        let mut moves = Vec::with_capacity(2);

        let all_occupied = *board.get_all_occupied();

        if board.active == Color::White {
            if board.white_queenside {
                let mov = CastleMove::QUEEN_WHITE;
                let mut nothing_inbetween = E1.get_between(A1);
                nothing_inbetween &= all_occupied;
                let nothing_inbetween = nothing_inbetween.bits == 0;
                let mut attacked_through_move = E1.get_between(C1);
                attacked_through_move |= mov.to;
                attacked_through_move &= forbidden;
                let attacked_through_move = attacked_through_move.bits != 0;

                if nothing_inbetween && !attacked_through_move {
                    moves.push(mov);
                }
            }

            if board.white_kingside {
                let mov = CastleMove::KING_WHITE;
                let mut nothing_inbetween = E1.get_between(H1);
                nothing_inbetween &= all_occupied;
                let nothing_inbetween = nothing_inbetween.bits == 0;
                let mut attacked_through_move = E1.get_between(G1);
                attacked_through_move |= mov.to;
                attacked_through_move &= forbidden;
                let attacked_through_move = attacked_through_move.bits != 0;

                if nothing_inbetween && !attacked_through_move {
                    moves.push(mov);
                }
            }
        } else if board.active == Color::Black {
            if board.black_queenside {
                let mov = CastleMove::QUEEN_BLACK;
                let mut nothing_inbetween = E8.get_between(A8);
                nothing_inbetween &= all_occupied;
                let nothing_inbetween = nothing_inbetween.bits == 0;

                let mut attacked_through_move = E8.get_between(C8);
                attacked_through_move |= mov.to;
                attacked_through_move &= forbidden;
                let attacked_through_move = attacked_through_move.bits != 0;

                if nothing_inbetween && !attacked_through_move {
                    moves.push(mov);
                }
            }

            if board.black_kingside {
                let mov = CastleMove::KING_BLACK;
                let mut nothing_inbetween = E8.get_between(H8);
                nothing_inbetween &= all_occupied;
                let nothing_inbetween = nothing_inbetween.bits == 0;
                let mut attacked_through_move = E8.get_between(G8);
                attacked_through_move |= mov.to;
                attacked_through_move &= forbidden;
                let attacked_through_move = attacked_through_move.bits != 0;

                if nothing_inbetween && !attacked_through_move {
                    moves.push(mov);
                }
            }
        }

        moves
    }

    fn get_single_king_moves(from: Square, forbidden: Bitboard) -> Bitboard {
        let mut moves = from.get_king_moves();
        moves ^= forbidden & moves;
        moves
    }

    fn get_knight_moves(
        board: &Board,
        pin_state: &PinState,
        forbidden: Bitboard,
        attackable: Bitboard,
    ) -> Vec<Move> {
        // TODO: This capacity might change but is here to make it more efficient.
        let mut moves = Vec::with_capacity(8);

        let squares = board.get_squares_by_piece(board.active, Piece::Knight);
        for from in squares {
            let moves_bb = Self::get_single_knight_moves(from, forbidden);
            let extracted =
                Self::extract_moves(Piece::Knight, from, pin_state, moves_bb, attackable);
            moves.extend(extracted);
        }

        moves
    }

    fn get_single_knight_moves(from: Square, forbidden: Bitboard) -> Bitboard {
        let mut moves = from.get_knight_moves();
        moves ^= forbidden & moves;
        moves
    }

    fn get_pawn_moves(
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
                let en_passant_move = Self::get_en_passant_move(board, pin_state, from, en_passant);
                if let Some(en_passant_move) = en_passant_move {
                    moves.push(en_passant_move);
                }
            }

            if !exclude_pawn_moves {
                let moves_bb = Self::get_single_pawn_moves(board, from, forbidden);
                let extracted =
                    Self::extract_moves(Piece::Pawn, from, pin_state, moves_bb, attackable);
                moves.extend(extracted);
            }

            let moves_bb =
                Self::get_single_pawn_attacks(board, include_pawn_attacks, from, forbidden);
            let extracted = Self::extract_moves(Piece::Pawn, from, pin_state, moves_bb, attackable);
            moves.extend(extracted);
        }

        moves
    }

    fn get_en_passant_move(
        board: &Board,
        pin_state: &PinState,
        from: Square,
        en_passant: EnPassant,
    ) -> Option<Move> {
        if pin_state.cant_en_passant {
            return None;
        }

        let attack_mask = from.get_pawn_attacks(board.active);
        let can_en_passant = attack_mask.is_set(en_passant.to_move);

        let pinned_allowed = pin_state.pins[from.index];
        let is_allowed = if pinned_allowed.bits != 0 {
            pinned_allowed.is_set(en_passant.to_capture)
        } else {
            true
        };

        if can_en_passant && is_allowed {
            let mov = EnPassantMove::new(from, en_passant.to_move, en_passant.to_capture);
            return Some(mov);
        }

        None
    }

    fn get_single_pawn_moves(board: &Board, from: Square, forbidden: Bitboard) -> Bitboard {
        let all_occupied = board.get_all_occupied();
        let from_bb: Bitboard = from.into();

        let push_mask = from.get_pawn_pushes(board.active);

        let attacking = all_occupied & push_mask;
        if attacking.bits != 0 {
            return Bitboard::default();
        }

        let mut moves = push_mask;
        moves ^= forbidden & moves;

        let double_push_allowed = (RANK_2 & from_bb) | (RANK_7 & from_bb);
        if double_push_allowed.bits == 0 {
            return moves;
        }

        let index = push_mask.get_trailing_index();
        let square = Square::index(index);
        let push_mask = square.get_pawn_pushes(board.active);

        let attacking = all_occupied & push_mask;
        if attacking.bits != 0 {
            return moves;
        }

        moves |= push_mask;
        moves ^= forbidden & moves;
        moves
    }

    fn get_single_pawn_attacks(
        board: &Board,
        include_unlegal_attacks: bool,
        from: Square,
        forbidden: Bitboard,
    ) -> Bitboard {
        let other_occupied = board.get_occupied(!board.active);

        let attack_mask = from.get_pawn_attacks(board.active);
        let mut moves = if include_unlegal_attacks {
            attack_mask
        } else {
            other_occupied & attack_mask
        };

        moves ^= forbidden & moves;
        moves
    }

    fn get_bishop_moves(
        board: &Board,
        pin_state: &PinState,
        forbidden: Bitboard,
        attackable: Bitboard,
    ) -> Vec<Move> {
        // TODO: This capacity might change but is here to make it more efficient.
        let mut moves = Vec::with_capacity(8);

        let all_occupied = *board.get_all_occupied();

        let squares = board.get_squares_by_piece(board.active, Piece::Bishop);
        for from in squares {
            let moves_bb = from.get_bishop_attacks(all_occupied) & !forbidden;
            let extracted =
                Self::extract_moves(Piece::Bishop, from, pin_state, moves_bb, attackable);
            moves.extend(extracted);
        }

        moves
    }

    fn get_rook_moves(
        board: &Board,
        pin_state: &PinState,
        forbidden: Bitboard,
        attackable: Bitboard,
    ) -> Vec<Move> {
        // TODO: This capacity might change but is here to make it more efficient.
        let mut moves = Vec::with_capacity(8);

        let all_occupied = *board.get_all_occupied();

        let squares = board.get_squares_by_piece(board.active, Piece::Rook);
        for from in squares {
            let moves_bb = from.get_rook_attacks(all_occupied) & !forbidden;
            let extracted = Self::extract_moves(Piece::Rook, from, pin_state, moves_bb, attackable);
            moves.extend(extracted);
        }

        moves
    }

    fn get_queen_moves(
        board: &Board,
        pin_state: &PinState,
        forbidden: Bitboard,
        attackable: Bitboard,
    ) -> Vec<Move> {
        // TODO: This capacity might change but is here to make it more efficient.
        let mut moves = Vec::with_capacity(8);

        let all_occupied = *board.get_all_occupied();

        let squares = board.get_squares_by_piece(board.active, Piece::Queen);
        for from in squares {
            let bishop_bb = from.get_bishop_attacks(all_occupied) & !forbidden;
            let rook_bb = from.get_rook_attacks(all_occupied) & !forbidden;
            let moves_bb = bishop_bb | rook_bb;

            let extracted =
                Self::extract_moves(Piece::Queen, from, pin_state, moves_bb, attackable);
            moves.extend(extracted);
        }

        moves
    }

    fn extract_squares(mut bitboard: Bitboard) -> Vec<Square> {
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
        piece: Piece,
        from: Square,
        pin_state: &PinState,
        moves_bb: Bitboard,
        attackable: Bitboard,
    ) -> Vec<Move> {
        // TODO: This capacity might change but is here to make it more efficient.
        let mut moves = Vec::with_capacity(8);

        let pinned_allowed = pin_state.pins[from.index];
        let promotion_ranks = match piece {
            Piece::Pawn => RANK_1 | RANK_8,
            _ => Bitboard::default(),
        };

        let squares = Self::extract_squares(moves_bb);
        for to in squares {
            if pinned_allowed.bits != 0 {
                let is_allowed = pinned_allowed.is_set(to);
                if !is_allowed {
                    continue;
                }
            }

            let is_promotion = promotion_ranks.is_set(to);
            let is_attack = attackable.is_set(to);

            if is_promotion {
                moves.push(PromotionMove::new(from, to, Piece::Queen, is_attack));
                moves.push(PromotionMove::new(from, to, Piece::Bishop, is_attack));
                moves.push(PromotionMove::new(from, to, Piece::Knight, is_attack));
                moves.push(PromotionMove::new(from, to, Piece::Rook, is_attack));
            } else if is_attack {
                moves.push(AttackMove::new(piece, from, to));
            } else {
                moves.push(NormalMove::new(piece, from, to));
            }
        }

        moves
    }
}
