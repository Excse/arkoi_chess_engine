use std::{
    fmt::Display,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not},
};

pub enum Direction {
    North,
    South,
    West,
    East,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bitboard {
    pub bits: u64,
}

impl Bitboard {
    pub const RANK_2: Bitboard = Bitboard::bits(65280);
    pub const RANK_7: Bitboard = Bitboard::bits(71776119061217280);

    pub const fn bits(bits: u64) -> Self {
        Self { bits }
    }

    pub fn square(rank: u8, file: u8) -> Self {
        let index = (rank * 8) + file;
        Bitboard::index(index as usize)
    }

    pub fn index(index: usize) -> Self {
        Self {
            bits: 1u64 << index,
        }
    }

    pub fn get_index(&self) -> u32 {
        self.bits.trailing_zeros()
    }

    pub fn get_rank_file(&self) -> (u8, char) {
        if self.bits == 0 {
            return (0, '0');
        }

        let index = self.bits.trailing_zeros();
        let rank = (index / 8) as u8 + 1;
        let file = (index % 8) as u8;
        let file = (b'a' + file) as char;
        (rank, file)
    }

    pub fn is_set(&self, square: Bitboard) -> bool {
        let new = self & square;
        new.bits != 0
    }

    pub fn shift(&mut self, direction: Direction) {
        match direction {
            Direction::South => self.bits >>= 8,
            Direction::North => self.bits <<= 8,
            Direction::West => self.bits <<= 1,
            Direction::East => self.bits >>= 1,
        }
    }
}

impl Display for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for rank in (0..8).rev() {
            write!(f, "  {}", rank + 1)?;

            for file in 0..8 {
                let square = Bitboard::square(rank, file);
                let is_set = self.is_set(square);
                let char = if is_set { "1" } else { "." };
                write!(f, " {}", char)?;
            }

            write!(f, "\n")?;
        }

        write!(f, "    a b c d e f g h\n")
    }
}

impl From<u64> for Bitboard {
    fn from(value: u64) -> Self {
        Bitboard::bits(value)
    }
}

impl BitXor for Bitboard {
    type Output = Bitboard;

    #[inline]
    fn bitxor(self, rhs: Bitboard) -> Self::Output {
        Bitboard::bits(self.bits ^ rhs.bits)
    }
}

impl BitXor<u64> for Bitboard {
    type Output = Bitboard;

    #[inline]
    fn bitxor(self, rhs: u64) -> Self::Output {
        Bitboard::bits(self.bits ^ rhs)
    }
}

impl BitXor<u64> for &Bitboard {
    type Output = Bitboard;

    #[inline]
    fn bitxor(self, rhs: u64) -> Self::Output {
        Bitboard::bits(self.bits ^ rhs)
    }
}

impl BitXorAssign for Bitboard {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Bitboard) {
        self.bits ^= rhs.bits;
    }
}

impl BitAnd for Bitboard {
    type Output = Bitboard;

    #[inline]
    fn bitand(self, rhs: Bitboard) -> Self::Output {
        Bitboard::bits(self.bits & rhs.bits)
    }
}

impl BitAnd<Bitboard> for &Bitboard {
    type Output = Bitboard;

    #[inline]
    fn bitand(self, rhs: Bitboard) -> Self::Output {
        Bitboard::bits(self.bits & rhs.bits)
    }
}

impl BitAnd<u64> for Bitboard {
    type Output = Bitboard;

    #[inline]
    fn bitand(self, rhs: u64) -> Self::Output {
        Bitboard::bits(self.bits & rhs)
    }
}

impl BitAnd<u64> for &Bitboard {
    type Output = Bitboard;

    #[inline]
    fn bitand(self, rhs: u64) -> Self::Output {
        Bitboard::bits(self.bits & rhs)
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.bits &= rhs.bits
    }
}

impl Not for Bitboard {
    type Output = Bitboard;

    fn not(self) -> Self::Output {
        Bitboard::bits(!self.bits)
    }
}

impl BitOr for Bitboard {
    type Output = Bitboard;

    #[inline]
    fn bitor(self, rhs: Bitboard) -> Self::Output {
        Bitboard::bits(self.bits | rhs.bits)
    }
}

impl BitOr<Bitboard> for &Bitboard {
    type Output = Bitboard;

    #[inline]
    fn bitor(self, rhs: Bitboard) -> Self::Output {
        Bitboard::bits(self.bits | rhs.bits)
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        self.bits |= rhs.bits;
    }
}

impl Not for &Bitboard {
    type Output = Bitboard;

    fn not(self) -> Self::Output {
        Bitboard::bits(!self.bits)
    }
}
