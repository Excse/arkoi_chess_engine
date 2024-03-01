use lookup::POLYGLOT_KEYS;

use crate::{
    board::{color::Color, piece::Piece, Board},
    square::Square,
    zobrist::ZobristHash,
};

pub const POLYGLOT_PIECE_OFFSET: usize = 0;
pub const POLYGLOT_CASTLING_OFFSET: usize = 768;
pub const POLYGLOT_EN_PASSANT_OFFSET: usize = 772;
pub const POLYGLOT_TURN_OFFSET: usize = 780;

pub struct PolyglotHasher;

impl PolyglotHasher {
    pub fn hash(board: &Board) -> ZobristHash {
        let mut hash = ZobristHash::new(0);

        for square_index in 0..64 {
            let square = Square::from_index(square_index);
            if let Some(tile) = board.get_tile(square) {
                hash ^= Self::piece_hash(tile.piece, tile.color, square);
            }
        }

        if board.active() == Color::White {
            hash ^= Self::turn_hash();
        }

        if board.can_white_kingside() {
            hash ^= Self::castling_hash(Color::White, true);
        }
        if board.can_white_queenside() {
            hash ^= Self::castling_hash(Color::White, false);
        }
        if board.can_black_kingside() {
            hash ^= Self::castling_hash(Color::Black, true);
        }
        if board.can_black_queenside() {
            hash ^= Self::castling_hash(Color::Black, false);
        }

        if let Some(en_passant) = &board.en_passant() {
            let adjacent_files = en_passant.to_capture.get_adjacent_files();
            let other_pawns = board.get_piece_board(board.active(), Piece::Pawn);
            let combined = adjacent_files & other_pawns;

            if !combined.is_empty() {
                hash ^= Self::en_passant_hash(en_passant.to_capture);
            }
        }

        hash
    }

    pub fn piece_hash(piece: Piece, color: Color, square: Square) -> ZobristHash {
        let kind_of_piece = ((piece.index() - 1) * 2) + color.index();
        let square = Square::from_index(square.index());

        let rank = square.rank() as usize;
        let file = square.file() as usize;

        let index = (64 * kind_of_piece) + 8 * rank + file;
        let key = POLYGLOT_KEYS[POLYGLOT_PIECE_OFFSET + index];

        ZobristHash::new(key)
    }

    pub fn castling_hash(color: Color, kingside: bool) -> ZobristHash {
        #[rustfmt::skip]
        let index = match color {
            Color::White => if kingside { 0 } else { 1 },
            Color::Black => if kingside { 2 } else { 3 },
        };
        let key = POLYGLOT_KEYS[POLYGLOT_CASTLING_OFFSET + index];

        ZobristHash::new(key)
    }

    pub fn en_passant_hash(square: Square) -> ZobristHash {
        let index = square.file() as usize;
        let key = POLYGLOT_KEYS[POLYGLOT_EN_PASSANT_OFFSET + index];

        ZobristHash::new(key)
    }

    pub fn turn_hash() -> ZobristHash {
        let key = POLYGLOT_KEYS[POLYGLOT_TURN_OFFSET];

        ZobristHash::new(key)
    }
}
