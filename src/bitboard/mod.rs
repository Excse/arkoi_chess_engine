pub mod constants;
pub mod error;
pub mod square;
mod tests;

use std::{
    fmt::{Display, LowerHex, UpperHex},
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not},
};

use self::square::Square;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bitboard {
    pub bits: u64,
}

impl Bitboard {
    pub const fn bits(bits: u64) -> Self {
        Self { bits }
    }

    pub const fn index(index: u8) -> Self {
        Self {
            bits: 1u64 << index,
        }
    }

    #[inline(always)]
    pub const fn get_trailing_index(&self) -> u8 {
        self.bits.trailing_zeros() as u8
    }

    #[inline(always)]
    pub const fn get_leading_index(&self) -> u8 {
        63 - self.bits.leading_zeros() as u8
    }

    #[inline]
    pub fn is_set(&self, other: impl Into<Bitboard>) -> bool {
        let other = other.into();
        (self & other).bits != 0
    }

    #[inline]
    pub fn pop_trailing(&mut self) -> Square {
        let index = self.get_trailing_index();
        let square = Square::index(index);
        // TODO: Make this better
        self.bits ^= Bitboard::from(square).bits;
        square
    }

    #[inline]
    pub fn pop_leading(&mut self) -> Square {
        let index = self.get_leading_index();
        let square = Square::index(index);
        // TODO: Make this better
        self.bits ^= Bitboard::from(square).bits;
        square
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
        write!(f, "0x{:016X}", self.bits)
    }
}

impl LowerHex for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016x}", self.bits)
    }
}

impl From<Square> for Bitboard {
    #[inline(always)]
    fn from(value: Square) -> Self {
        Bitboard::index(value.index as u8)
    }
}

impl From<u64> for Bitboard {
    #[inline(always)]
    fn from(value: u64) -> Self {
        Bitboard::bits(value)
    }
}

impl BitXor for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn bitxor(self, rhs: Bitboard) -> Self::Output {
        Bitboard::bits(self.bits ^ rhs.bits)
    }
}

impl BitXor<u64> for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn bitxor(self, rhs: u64) -> Self::Output {
        Bitboard::bits(self.bits ^ rhs)
    }
}

impl BitXor<Square> for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn bitxor(self, rhs: Square) -> Self::Output {
        let rhs: Bitboard = rhs.into();
        Bitboard::bits(self.bits ^ rhs.bits)
    }
}

impl BitXor<u64> for &Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn bitxor(self, rhs: u64) -> Self::Output {
        Bitboard::bits(self.bits ^ rhs)
    }
}

impl BitXorAssign for Bitboard {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Bitboard) {
        self.bits ^= rhs.bits;
    }
}

impl BitXorAssign<Square> for Bitboard {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Square) {
        let rhs: Bitboard = rhs.into();
        self.bits ^= rhs.bits
    }
}

impl BitAnd<u64> for Square {
    type Output = Bitboard;

    #[inline(always)]
    fn bitand(self, rhs: u64) -> Self::Output {
        let lhs: Bitboard = self.into();
        let rhs: Bitboard = rhs.into();
        Bitboard::bits(lhs.bits & rhs.bits)
    }
}

impl BitAnd for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn bitand(self, rhs: Bitboard) -> Self::Output {
        Bitboard::bits(self.bits & rhs.bits)
    }
}

impl BitAnd<Square> for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn bitand(self, rhs: Square) -> Self::Output {
        let rhs: Bitboard = rhs.into();
        Bitboard::bits(self.bits & rhs.bits)
    }
}

impl BitAnd<Bitboard> for &Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn bitand(self, rhs: Bitboard) -> Self::Output {
        Bitboard::bits(self.bits & rhs.bits)
    }
}

impl BitAnd<&Bitboard> for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn bitand(self, rhs: &Bitboard) -> Self::Output {
        Bitboard::bits(self.bits & rhs.bits)
    }
}

impl BitAnd<Square> for &Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn bitand(self, rhs: Square) -> Self::Output {
        let rhs: Bitboard = rhs.into();
        Bitboard::bits(self.bits & rhs.bits)
    }
}

impl BitAnd<&Bitboard> for u64 {
    type Output = Bitboard;

    #[inline(always)]
    fn bitand(self, rhs: &Bitboard) -> Self::Output {
        Bitboard::bits(self & rhs.bits)
    }
}

impl BitAnd<u64> for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn bitand(self, rhs: u64) -> Self::Output {
        Bitboard::bits(self.bits & rhs)
    }
}

impl BitAnd<u64> for &Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn bitand(self, rhs: u64) -> Self::Output {
        Bitboard::bits(self.bits & rhs)
    }
}

impl BitAnd<Bitboard> for u64 {
    type Output = Bitboard;

    #[inline(always)]
    fn bitand(self, rhs: Bitboard) -> Self::Output {
        Bitboard::bits(self & rhs.bits)
    }
}

impl BitAndAssign for Bitboard {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        self.bits &= rhs.bits
    }
}

impl BitAndAssign<u64> for Bitboard {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: u64) {
        self.bits &= rhs
    }
}

impl Not for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn not(self) -> Self::Output {
        Bitboard::bits(!self.bits)
    }
}

impl BitOr for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn bitor(self, rhs: Bitboard) -> Self::Output {
        Bitboard::bits(self.bits | rhs.bits)
    }
}

impl BitOr<Bitboard> for &Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn bitor(self, rhs: Bitboard) -> Self::Output {
        Bitboard::bits(self.bits | rhs.bits)
    }
}

impl BitOrAssign for Bitboard {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        self.bits |= rhs.bits;
    }
}

impl BitOrAssign<u64> for Bitboard {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: u64) {
        self.bits |= rhs;
    }
}

impl BitOrAssign<Square> for Bitboard {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Square) {
        let rhs: Bitboard = rhs.into();
        self.bits |= rhs.bits
    }
}

impl Not for &Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn not(self) -> Self::Output {
        Bitboard::bits(!self.bits)
    }
}
