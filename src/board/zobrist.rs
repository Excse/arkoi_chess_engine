use std::{
    fmt::{Binary, LowerHex, Octal, UpperHex},
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign},
};

use rand::Rng;

use crate::{bitboard::square::Square, board::color::Color};

use super::{piece::Piece, Board};

#[derive(Default, Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct ZobristHash(u64);

impl ZobristHash {
    pub const fn new(hash: u64) -> ZobristHash {
        ZobristHash(hash)
    }

    #[inline(always)]
    pub const fn hash(&self) -> u64 {
        self.0
    }
}

impl LowerHex for ZobristHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl UpperHex for ZobristHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", self.0)
    }
}

impl Octal for ZobristHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:o}", self.0)
    }
}

impl Binary for ZobristHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:b}", self.0)
    }
}

impl<T: Into<ZobristHash>> BitAnd<T> for ZobristHash {
    type Output = ZobristHash;

    #[inline(always)]
    fn bitand(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        ZobristHash::new(self.0 & rhs.0)
    }
}

impl<T: Into<ZobristHash>> BitAnd<T> for &ZobristHash {
    type Output = ZobristHash;

    #[inline(always)]
    fn bitand(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        ZobristHash::new(self.0 & rhs.0)
    }
}

impl<T: Into<ZobristHash>> BitAndAssign<T> for ZobristHash {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.0 &= rhs.0;
    }
}

impl<T: Into<ZobristHash>> BitAndAssign<T> for &mut ZobristHash {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.0 &= rhs.0;
    }
}

impl<T: Into<ZobristHash>> BitOr<T> for ZobristHash {
    type Output = ZobristHash;

    #[inline(always)]
    fn bitor(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        ZobristHash::new(self.0 | rhs.0)
    }
}

impl<T: Into<ZobristHash>> BitOr<T> for &ZobristHash {
    type Output = ZobristHash;

    #[inline(always)]
    fn bitor(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        ZobristHash::new(self.0 | rhs.0)
    }
}

impl<T: Into<ZobristHash>> BitOrAssign<T> for ZobristHash {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.0 |= rhs.0;
    }
}

impl<T: Into<ZobristHash>> BitOrAssign<T> for &mut ZobristHash {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.0 |= rhs.0;
    }
}

impl<T: Into<ZobristHash>> BitXor<T> for ZobristHash {
    type Output = ZobristHash;

    #[inline(always)]
    fn bitxor(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        ZobristHash::new(self.0 ^ rhs.0)
    }
}

impl<T: Into<ZobristHash>> BitXor<T> for &ZobristHash {
    type Output = ZobristHash;

    #[inline(always)]
    fn bitxor(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        ZobristHash::new(self.0 ^ rhs.0)
    }
}

impl<T: Into<ZobristHash>> BitXorAssign<T> for ZobristHash {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.0 ^= rhs.0;
    }
}

impl<T: Into<ZobristHash>> BitXorAssign<T> for &mut ZobristHash {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.0 ^= rhs.0;
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
            let square = Square::from_index(square_index);
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
