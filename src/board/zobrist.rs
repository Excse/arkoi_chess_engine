use std::ops::{BitXor, BitXorAssign};

use rand::Rng;

use crate::{bitboard::square::Square, board::color::Color};

use super::{piece::Piece, Board};

#[derive(Default, Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct ZobristHash(pub u64);

impl ZobristHash {
    pub const fn new(hash: u64) -> ZobristHash {
        ZobristHash(hash)
    }
}

impl BitXor for ZobristHash {
    type Output = Self;

    #[inline(always)]
    fn bitxor(self, other: Self) -> Self::Output {
        ZobristHash::new(self.0 ^ other.0)
    }
}

impl BitXorAssign for ZobristHash {
    #[inline(always)]
    fn bitxor_assign(&mut self, other: Self) {
        self.0 ^= other.0;
    }
}

#[derive(Debug)]
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
        let mut hash = ZobristHash(0);

        for square_index in 0..64 {
            let square = Square::by_index(square_index);
            if let Some(colored_piece) = board.get_piece_type(square) {
                hash ^= self.piece_hash(colored_piece.piece, colored_piece.color, square);
            }
        }

        if board.gamestate.active == Color::Black {
            hash ^= self.side_hash();
        }

        if board.gamestate.white_kingside {
            hash ^= self.castling[0];
        }
        if board.gamestate.white_queenside {
            hash ^= self.castling[1];
        }
        if board.gamestate.black_kingside {
            hash ^= self.castling[2];
        }
        if board.gamestate.black_queenside {
            hash ^= self.castling[3];
        }

        if let Some(en_passant) = &board.gamestate.en_passant {
            hash ^= self.en_passant_hash(en_passant.to_capture);
        }

        hash
    }

    pub fn piece_hash(&self, piece: Piece, color: Color, square: Square) -> ZobristHash {
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
        unsafe {
            let hash = self.depth.get_unchecked(depth as usize);
            *hash
        }
    }

    pub fn side_hash(&self) -> ZobristHash {
        self.side
    }
}
