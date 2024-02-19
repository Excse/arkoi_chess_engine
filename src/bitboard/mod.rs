pub mod constants;
pub mod error;
pub mod square;
mod tests;

use std::{
    fmt::{Binary, Display, LowerHex, Octal, UpperHex},
    ops::{
        BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, ShlAssign, Shr,
        ShrAssign,
    },
};

use self::square::Square;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bitboard(u64);

impl Bitboard {
    pub const ALL_BITS: Bitboard = Bitboard(0xFFFFFFFFFFFFFFFF);
    pub const EMPTY: Bitboard = Bitboard(0);

    pub const fn from_bits(bits: u64) -> Self {
        Self(bits)
    }

    pub const fn from_index(index: u8) -> Self {
        debug_assert!(index < 64);

        Self(1u64 << index)
    }

    #[inline(always)]
    pub const fn bits(&self) -> u64 {
        self.0
    }

    #[inline(always)]
    pub const fn get_trailing_index(&self) -> u8 {
        self.0.trailing_zeros() as u8
    }

    #[inline(always)]
    pub const fn get_leading_index(&self) -> u8 {
        63 - self.0.leading_zeros() as u8
    }

    #[inline]
    pub fn is_set(&self, other: impl Into<Bitboard>) -> bool {
        let other = other.into();
        let combined = self & other;
        !combined.is_empty()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn count_ones(&self) -> usize {
        self.0.count_ones() as usize
    }

    #[inline]
    pub fn pop_trailing(&mut self) -> Square {
        let index = self.get_trailing_index();
        let square = Square::from(index);
        // TODO: Make this better
        self.0 ^= Bitboard::from(square).0;
        square
    }

    #[inline]
    pub fn pop_leading(&mut self) -> Square {
        let index = self.get_leading_index();
        let square = Square::from(index);
        // TODO: Make this better
        self.0 ^= Bitboard::from(square).0;
        square
    }

    #[inline(always)]
    pub fn get_magic_index(&self, magic: u64, ones: usize) -> usize {
        (self.0.wrapping_mul(magic) >> (64 - ones)) as usize
    }

    #[inline(always)]
    pub fn is_magic_canidate(&self, magic: u64) -> bool {
        let candidate = self.0.wrapping_mul(magic) & 0xFF00000000000000;
        candidate.count_ones() >= 6
    }

    pub fn get_squares(&self) -> Vec<Square> {
        let mut squares = Vec::with_capacity(self.count_ones());

        for square in *self {
            squares.push(square);
        }

        squares
    }
}

impl Iterator for Bitboard {
    type Item = Square;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            return None;
        }

        Some(self.pop_trailing())
    }
}

impl Display for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for rank in (0..8).rev() {
            write!(f, "  {}", rank + 1)?;

            for file in 0..8 {
                let square = Square::new(rank, file);
                let is_set = self.is_set(square);
                let char = if is_set { "1" } else { "." };
                write!(f, " {}", char)?;
            }

            write!(f, "\n")?;
        }

        write!(f, "    a b c d e f g h\n")
    }
}

impl UpperHex for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        UpperHex::fmt(&self.0, f)
    }
}

impl LowerHex for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        LowerHex::fmt(&self.0, f)
    }
}

impl Octal for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Octal::fmt(&self.0, f)
    }
}

impl Binary for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Binary::fmt(&self.0, f)
    }
}

impl From<Square> for Bitboard {
    #[inline(always)]
    fn from(value: Square) -> Self {
        Bitboard::from_index(u8::from(value))
    }
}

impl From<u64> for Bitboard {
    #[inline(always)]
    fn from(value: u64) -> Self {
        Bitboard::from_bits(value)
    }
}

impl From<&Bitboard> for Bitboard {
    #[inline(always)]
    fn from(value: &Bitboard) -> Self {
        Bitboard::from_bits(value.0)
    }
}

impl<T: Into<Bitboard>> BitAnd<T> for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn bitand(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Bitboard::from_bits(self.0 & rhs.0)
    }
}

impl<T: Into<Bitboard>> BitAnd<T> for &Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn bitand(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Bitboard::from_bits(self.0 & rhs.0)
    }
}

impl<T: Into<Bitboard>> BitAndAssign<T> for Bitboard {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.0 &= rhs.0;
    }
}

impl<T: Into<Bitboard>> BitAndAssign<T> for &mut Bitboard {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.0 &= rhs.0;
    }
}

impl<T: Into<Bitboard>> BitOr<T> for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn bitor(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Bitboard::from_bits(self.0 | rhs.0)
    }
}

impl<T: Into<Bitboard>> BitOr<T> for &Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn bitor(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Bitboard::from_bits(self.0 | rhs.0)
    }
}

impl<T: Into<Bitboard>> BitOrAssign<T> for Bitboard {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.0 |= rhs.0;
    }
}

impl<T: Into<Bitboard>> BitOrAssign<T> for &mut Bitboard {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.0 |= rhs.0;
    }
}

impl<T: Into<Bitboard>> BitXor<T> for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn bitxor(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Bitboard::from_bits(self.0 ^ rhs.0)
    }
}

impl<T: Into<Bitboard>> BitXor<T> for &Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn bitxor(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Bitboard::from_bits(self.0 ^ rhs.0)
    }
}

impl<T: Into<Bitboard>> BitXorAssign<T> for Bitboard {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.0 ^= rhs.0;
    }
}

impl<T: Into<Bitboard>> BitXorAssign<T> for &mut Bitboard {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.0 ^= rhs.0;
    }
}

impl<T: Into<Bitboard>> Shl<T> for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn shl(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Bitboard::from_bits(self.0 << rhs.0)
    }
}

impl<T: Into<Bitboard>> Shl<T> for &Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn shl(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Bitboard::from_bits(self.0 << rhs.0)
    }
}

impl<T: Into<Bitboard>> ShlAssign<T> for Bitboard {
    #[inline(always)]
    fn shl_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.0 <<= rhs.0;
    }
}

impl<T: Into<Bitboard>> ShlAssign<T> for &mut Bitboard {
    #[inline(always)]
    fn shl_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.0 <<= rhs.0;
    }
}

impl<T: Into<Bitboard>> Shr<T> for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn shr(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Bitboard::from_bits(self.0 >> rhs.0)
    }
}

impl<T: Into<Bitboard>> Shr<T> for &Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn shr(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Bitboard::from_bits(self.0 >> rhs.0)
    }
}

impl<T: Into<Bitboard>> ShrAssign<T> for Bitboard {
    #[inline(always)]
    fn shr_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.0 >>= rhs.0;
    }
}

impl<T: Into<Bitboard>> ShrAssign<T> for &mut Bitboard {
    #[inline(always)]
    fn shr_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.0 >>= rhs.0;
    }
}

impl Not for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn not(self) -> Self::Output {
        Bitboard::from_bits(!self.0)
    }
}

impl Not for &Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn not(self) -> Self::Output {
        Bitboard::from_bits(!self.0)
    }
}
