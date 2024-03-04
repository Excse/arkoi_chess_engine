use rand::Rng;

use crate::{
    board::{color::Color, piece::Piece, Board},
    square::Square,
};

use super::hash::ZobristHash;

pub type PieceKeys = [[[ZobristHash; Board::SIZE]; Piece::COUNT]; Color::COUNT];
pub type EnPassantKey = [ZobristHash; 8];
pub type CastlingKey = [ZobristHash; 4];
pub type DepthKeys = [ZobristHash; 64];
pub type TurnKey = ZobristHash;

#[derive(Debug, Clone)]
pub struct ZobristHasher {
    piece_keys: PieceKeys,
    turn_key: TurnKey,
    castling_keys: CastlingKey,
    en_passant_keys: EnPassantKey,
    depth_keys: DepthKeys,
}

impl ZobristHasher {
    pub fn random<T: Rng>(rand: &mut T) -> Self {
        let mut piece_keys = [[[ZobristHash::default(); Board::SIZE]; Piece::COUNT]; Color::COUNT];
        for color in 0..Color::COUNT {
            for piece in 0..Piece::COUNT {
                for square in 0..Board::SIZE {
                    piece_keys[color][piece][square] = ZobristHash::new(rand.next_u64());
                }
            }
        }

        let turn_key = ZobristHash::new(rand.next_u64());

        let mut castling_keys = [ZobristHash::default(); 4];
        for index in 0..4 {
            castling_keys[index] = ZobristHash::new(rand.next_u64());
        }

        let mut en_passant_keys = [ZobristHash::default(); 8];
        for index in 0..8 {
            en_passant_keys[index] = ZobristHash::new(rand.next_u64());
        }

        let mut depth_keys = [ZobristHash::default(); 64];
        for index in 0..32 {
            depth_keys[index] = ZobristHash::new(rand.next_u64());
        }

        Self {
            piece_keys,
            turn_key,
            castling_keys,
            en_passant_keys,
            depth_keys,
        }
    }

    pub fn hash(&self, board: &Board) -> ZobristHash {
        let mut hash = ZobristHash::new(0);

        for square_index in 0..64 {
            let square = Square::from_index(square_index);
            if let Some(tile) = board.get_tile(square) {
                hash ^= self.piece_hash(tile.piece, tile.color, square);
            }
        }

        if board.active() == Color::White {
            hash ^= self.turn_hash();
        }

        if board.can_white_kingside() {
            hash ^= self.castling_hash(Color::White, true);
        }
        if board.can_white_queenside() {
            hash ^= self.castling_hash(Color::White, false);
        }
        if board.can_black_kingside() {
            hash ^= self.castling_hash(Color::Black, true);
        }
        if board.can_black_queenside() {
            hash ^= self.castling_hash(Color::Black, false);
        }

        if let Some(en_passant) = &board.en_passant() {
            hash ^= self.en_passant_hash(en_passant.to_capture);
        }

        hash
    }

    pub fn piece_hash(&self, piece: Piece, color: Color, square: Square) -> ZobristHash {
        debug_assert!(color.index() < Color::COUNT);
        debug_assert!(piece.index() < Piece::COUNT);
        debug_assert!(piece != Piece::None);

        unsafe {
            let pieces = self.piece_keys.get_unchecked(color.index());
            let squares = pieces.get_unchecked(piece.index());
            let hash = squares.get_unchecked(usize::from(square));
            *hash
        }
    }

    pub fn en_passant_hash(&self, square: Square) -> ZobristHash {
        unsafe {
            let file_index = square.file() as usize;
            let hash = self.en_passant_keys.get_unchecked(file_index);
            *hash
        }
    }

    pub fn depth_hash(&self, depth: u8) -> ZobristHash {
        debug_assert!(depth < 64);

        unsafe {
            let hash = self.depth_keys.get_unchecked(depth as usize);
            *hash
        }
    }

    pub fn castling_hash(&self, color: Color, kingside: bool) -> ZobristHash {
        #[rustfmt::skip]
        let index = match color {
            Color::White => if kingside { 0 } else { 1 },
            Color::Black => if kingside { 2 } else { 3 },
        };

        unsafe {
            let hash = self.castling_keys.get_unchecked(index);
            *hash
        }
    }

    pub fn turn_hash(&self) -> ZobristHash {
        self.turn_key
    }
}
