use std::fmt::Display;

use crate::{
    bitboard::Bitboard,
    board::{Board, BoardError, Piece},
    tables::{KING_MOVES, KNIGHT_MOVES, PAWN_ATTACKS, PAWN_PUSHES, RAYS},
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

        let pawn_moves = self.get_pawn_moves();
        moves.extend(pawn_moves);

        let king_moves = self.get_king_moves();
        moves.extend(king_moves);

        let knight_moves = self.get_knight_moves();
        moves.extend(knight_moves);

        let bishop_moves = self.get_bishop_moves();
        moves.extend(bishop_moves);

        let rook_moves = self.get_rook_moves();
        moves.extend(rook_moves);

        let queen_moves = self.get_queen_moves();
        moves.extend(queen_moves);

        Ok(moves)
    }

    fn get_king_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();

        let mut kings = *self.board.get_active_piece_board(Piece::King);
        while kings.bits != 0 {
            let index = kings.bits.trailing_zeros() as usize;
            let from_bb = Bitboard::index(index);
            kings ^= from_bb;

            let moves_bb = self.get_single_king_moves(index);
            let extracted = self.extract_moves(Piece::King, from_bb, moves_bb);
            moves.extend(extracted);
        }

        moves
    }

    fn get_single_king_moves(&self, index: usize) -> Bitboard {
        let own_unoccupied = !self.board.get_own_occupied();

        let moves_mask = KING_MOVES[index];
        own_unoccupied & moves_mask
    }

    fn get_knight_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();

        let mut knights = *self.board.get_active_piece_board(Piece::Knight);
        while knights.bits != 0 {
            let index = knights.bits.trailing_zeros() as usize;
            let from_bb = Bitboard::index(index);
            knights ^= from_bb;

            let moves_bb = self.get_single_knight_moves(index);
            let extracted = self.extract_moves(Piece::Knight, from_bb, moves_bb);
            moves.extend(extracted);
        }

        moves
    }

    fn get_single_knight_moves(&self, index: usize) -> Bitboard {
        let own_unoccupied = !self.board.get_own_occupied();

        let moves_mask = KNIGHT_MOVES[index];
        own_unoccupied & moves_mask
    }

    fn get_pawn_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();

        let mut pawns = *self.board.get_active_piece_board(Piece::Pawn);
        while pawns.bits != 0 {
            let index = pawns.bits.trailing_zeros() as usize;
            let from_bb = Bitboard::index(index);
            pawns ^= from_bb;

            let moves_bb = self.get_single_pawn_moves(index, from_bb);
            let extracted = self.extract_moves(Piece::Pawn, from_bb, moves_bb);
            moves.extend(extracted);

            let moves_bb = self.get_single_pawn_attacks(index);
            let extracted = self.extract_moves(Piece::Pawn, from_bb, moves_bb);
            moves.extend(extracted);
        }

        moves
    }

    fn get_single_pawn_moves(&self, index: usize, from_bb: Bitboard) -> Bitboard {
        let active_index = self.board.active.index();
        let all_occupied = self.board.get_all_occupied();

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

    fn get_single_pawn_attacks(&self, index: usize) -> Bitboard {
        let other_occupied = self.board.get_other_occpuied();
        let active_index = self.board.active.index();

        let attack_mask = PAWN_ATTACKS[active_index][index];
        other_occupied & attack_mask
    }

    fn get_bishop_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();

        let mut bishops = *self.board.get_active_piece_board(Piece::Bishop);
        while bishops.bits != 0 {
            let index = bishops.bits.trailing_zeros() as usize;
            let from_bb = Bitboard::index(index);
            bishops ^= from_bb;

            let moves_bb = self.get_single_bishop_moves(index);
            let extracted = self.extract_moves(Piece::Bishop, from_bb, moves_bb);
            moves.extend(extracted);
        }

        moves
    }

    fn get_single_bishop_moves(&self, index: usize) -> Bitboard {
        let own_occupied = self.board.get_own_occupied();
        let all_occupied = self.board.get_all_occupied();
        let mut moves = Bitboard::default();

        let north_east_ray = RAYS[index][2];
        moves |= north_east_ray;
        let blocking = north_east_ray & all_occupied;
        if blocking.bits != 0 {
            let blocker_index = blocking.bits.trailing_zeros() as usize;
            moves &= !RAYS[blocker_index][2];
        }

        let south_east_ray = RAYS[index][7];
        moves |= south_east_ray;
        let blocking = south_east_ray & all_occupied;
        if blocking.bits != 0 {
            let blocker_index = 63 - blocking.bits.leading_zeros() as usize;
            moves &= !RAYS[blocker_index][7];
        }

        let south_west_ray = RAYS[index][5];
        moves |= south_west_ray;
        let blocking = south_west_ray & all_occupied;
        if blocking.bits != 0 {
            let blocker_index = 63 - blocking.bits.leading_zeros() as usize;
            moves &= !RAYS[blocker_index][5];
        }

        let north_west_ray = RAYS[index][0];
        moves |= north_west_ray;
        let blocking = north_west_ray & all_occupied;
        if blocking.bits != 0 {
            let blocker_index = blocking.bits.trailing_zeros() as usize;
            moves &= !RAYS[blocker_index][0];
        }

        moves ^= own_occupied & moves;
        moves
    }

    fn get_rook_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();

        let mut rooks = *self.board.get_active_piece_board(Piece::Rook);
        while rooks.bits != 0 {
            let index = rooks.bits.trailing_zeros() as usize;
            let from_bb = Bitboard::index(index);
            rooks ^= from_bb;

            let moves_bb = self.get_single_rook_moves(index);
            let extracted = self.extract_moves(Piece::Rook, from_bb, moves_bb);
            moves.extend(extracted);
        }

        moves
    }

    fn get_single_rook_moves(&self, index: usize) -> Bitboard {
        let own_occupied = self.board.get_own_occupied();
        let all_occupied = self.board.get_all_occupied();
        let mut moves = Bitboard::default();

        let north_ray = RAYS[index][1];
        moves |= north_ray;
        let blocking = north_ray & all_occupied;
        if blocking.bits != 0 {
            let blocker_index = blocking.bits.trailing_zeros() as usize;
            moves &= !RAYS[blocker_index][1];
        }

        let east_ray = RAYS[index][4];
        moves |= east_ray;
        let blocking = east_ray & all_occupied;
        if blocking.bits != 0 {
            let blocker_index = blocking.bits.trailing_zeros() as usize;
            moves &= !RAYS[blocker_index][4];
        }

        let south_ray = RAYS[index][6];
        moves |= south_ray;
        let blocking = south_ray & all_occupied;
        if blocking.bits != 0 {
            let blocker_index = 63 - blocking.bits.leading_zeros() as usize;
            moves &= !RAYS[blocker_index][6];
        }

        let west_ray = RAYS[index][3];
        moves |= west_ray;
        let blocking = west_ray & all_occupied;
        if blocking.bits != 0 {
            let blocker_index = 63 - blocking.bits.leading_zeros() as usize;
            moves &= !RAYS[blocker_index][3];
        }

        moves ^= own_occupied & moves;
        moves
    }

    fn get_queen_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();

        let mut queens = *self.board.get_active_piece_board(Piece::Queen);
        while queens.bits != 0 {
            let index = queens.bits.trailing_zeros() as usize;
            let from_bb = Bitboard::index(index);
            queens ^= from_bb;

            let bishop_bb = self.get_single_bishop_moves(index);
            let rook_bb = self.get_single_rook_moves(index);
            let moves_bb = bishop_bb | rook_bb;
            let extracted = self.extract_moves(Piece::Queen, from_bb, moves_bb);
            moves.extend(extracted);
        }

        moves
    }

    fn extract_moves(&self, piece: Piece, from: Bitboard, mut moves_bb: Bitboard) -> Vec<Move> {
        let mut moves = Vec::new();

        let other_occupied = self.board.get_other_occpuied();
        while moves_bb.bits != 0 {
            let to_index = moves_bb.bits.trailing_zeros() as usize;
            let to = Bitboard::index(to_index);
            moves_bb ^= to;

            let is_attack = other_occupied & to;
            let is_attack = is_attack.bits != 0;

            let mov = Move::new(piece, is_attack, from, to);
            moves.push(mov);
        }

        moves
    }
}
