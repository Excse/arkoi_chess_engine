use rand::Rng;

use crate::{
    board::{color::Color, piece::Piece, Board},
    square::Square,
};

use super::hash::ZobristHash;

#[derive(Debug, Clone)]
pub struct ZobristHasher {
    pieces: [[[ZobristHash; Board::SIZE]; Piece::COUNT]; Color::COUNT],
    side: ZobristHash,
    // TODO: Change this
    pub castling: [ZobristHash; 4],
    en_passant: [ZobristHash; 8],
    depth: [ZobristHash; 32],
}

impl ZobristHasher {
    pub fn new<T: Rng>(rand: &mut T) -> ZobristHasher {
        let mut pieces = [[[ZobristHash::default(); Board::SIZE]; Piece::COUNT]; Color::COUNT];
        for color in 0..Color::COUNT {
            for piece in 0..Piece::COUNT {
                for square in 0..Board::SIZE {
                    pieces[color][piece][square] = ZobristHash::new(rand.next_u64());
                }
            }
        }

        let side = ZobristHash::new(rand.next_u64());

        let mut castling = [ZobristHash::default(); 4];
        for index in 0..4 {
            castling[index] = ZobristHash::new(rand.next_u64());
        }

        let mut en_passant = [ZobristHash::default(); 8];
        for index in 0..8 {
            en_passant[index] = ZobristHash::new(rand.next_u64());
        }

        let mut depth = [ZobristHash::default(); 32];
        for index in 0..32 {
            depth[index] = ZobristHash::new(rand.next_u64());
        }

        ZobristHasher {
            pieces,
            side,
            castling,
            en_passant,
            depth,
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

        if board.active() == Color::Black {
            hash ^= self.side_hash();
        }

        if board.can_white_kingside() {
            hash ^= self.castling[0];
        }
        if board.can_white_queenside() {
            hash ^= self.castling[1];
        }
        if board.can_black_kingside() {
            hash ^= self.castling[2];
        }
        if board.can_black_queenside() {
            hash ^= self.castling[3];
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
            let pieces = self.pieces.get_unchecked(color.index());
            let squares = pieces.get_unchecked(piece.index());
            let hash = squares.get_unchecked(usize::from(square));
            *hash
        }
    }

    pub fn en_passant_hash(&self, square: Square) -> ZobristHash {
        unsafe {
            let file_index = square.file() as usize;
            let hash = self.en_passant.get_unchecked(file_index);
            *hash
        }
    }

    pub fn depth_hash(&self, depth: u8) -> ZobristHash {
        debug_assert!(depth < 32);

        unsafe {
            let hash = self.depth.get_unchecked(depth as usize);
            *hash
        }
    }

    pub fn side_hash(&self) -> ZobristHash {
        self.side
    }
}
