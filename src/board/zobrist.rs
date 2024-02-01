use std::ops::{BitXor, BitXorAssign};

use rand::Rng;

use crate::{bitboard::square::Square, board::color::Color};

use super::{piece::Piece, Board};

#[derive(Default, Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct ZobristHash(pub u64);

impl ZobristHash {
    pub fn new(hash: u64) -> ZobristHash {
        ZobristHash(hash)
    }
}

impl BitXor for ZobristHash {
    type Output = Self;

    fn bitxor(self, other: Self) -> Self::Output {
        ZobristHash::new(self.0 ^ other.0)
    }
}

impl BitXorAssign for ZobristHash {
    fn bitxor_assign(&mut self, other: Self) {
        self.0 ^= other.0;
    }
}

#[derive(Debug)]
pub struct ZobristHasher {
    pub pieces: [[ZobristHash; 64]; 12],
    pub side: ZobristHash,
    pub castling: [ZobristHash; 4],
    pub en_passant: [ZobristHash; 8],
}

impl ZobristHasher {
    pub fn new<T: Rng>(rand: &mut T) -> ZobristHasher {
        let mut pieces: [[ZobristHash; 64]; 12] = [[ZobristHash::default(); 64]; 12];
        for i in 0..12 {
            for j in 0..64 {
                pieces[i][j] = ZobristHash::new(rand.next_u64());
            }
        }

        let side = ZobristHash::new(rand.next_u64());

        let mut castling: [ZobristHash; 4] = [ZobristHash::default(); 4];
        for i in 0..4 {
            castling[i] = ZobristHash::new(rand.next_u64());
        }

        let mut en_passant: [ZobristHash; 8] = [ZobristHash::default(); 8];
        for i in 0..8 {
            en_passant[i] = ZobristHash::new(rand.next_u64());
        }

        ZobristHasher {
            pieces,
            side,
            castling,
            en_passant,
        }
    }

    pub fn hash(&self, board: &Board) -> ZobristHash {
        let mut hash = ZobristHash(0);

        for square_index in 0..64 {
            let square = Square::index(square_index);
            if let Some(piece) = board.get_colored_piece_type(square) {
                let zobrist_index = piece.piece.index() * (piece.color.index() + 1);
                hash ^= self.pieces[zobrist_index][square_index];
            }
        }

        if board.active == Color::Black {
            hash ^= self.side;
        }

        if board.white_kingside {
            hash ^= self.castling[0];
        }
        if board.white_queenside {
            hash ^= self.castling[1];
        }
        if board.black_kingside {
            hash ^= self.castling[2];
        }
        if board.black_queenside {
            hash ^= self.castling[3];
        }

        if let Some(en_passant) = board.en_passant {
            let to_capture = en_passant.to_capture;
            let file_index = to_capture.file() as usize;
            hash ^= self.en_passant[file_index];
        }

        hash
    }

    pub fn get_piece_hash(&self, piece: Piece, color: Color, square: Square) -> ZobristHash {
        let zobrist_index = piece.index() * (color.index() + 1);
        self.pieces[zobrist_index][square.index]
    }
}
