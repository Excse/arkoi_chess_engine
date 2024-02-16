pub mod constants;
pub mod error;
pub mod square;
mod tests;

use std::{
    fmt::{Binary, Display, LowerHex, Octal, UpperHex},
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
        let square = Square::from(index);
        // TODO: Make this better
        self.bits ^= Bitboard::from(square).bits;
        square
    }

    #[inline]
    pub fn pop_leading(&mut self) -> Square {
        let index = self.get_leading_index();
        let square = Square::from(index);
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
        UpperHex::fmt(&self.bits, f)
    }
}

impl LowerHex for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        LowerHex::fmt(&self.bits, f)
    }
}

impl Octal for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Octal::fmt(&self.bits, f)
    }
}

impl Binary for Bitboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Binary::fmt(&self.bits, f)
    }
}

impl From<Square> for Bitboard {
    #[inline(always)]
    fn from(value: Square) -> Self {
        Bitboard::index(u8::from(value))
    }
}

impl From<u64> for Bitboard {
    #[inline(always)]
    fn from(value: u64) -> Self {
        Bitboard::bits(value)
    }
}

impl From<&Bitboard> for Bitboard {
    #[inline(always)]
    fn from(value: &Bitboard) -> Self {
        Bitboard::bits(value.bits)
    }
}

impl<T: Into<Bitboard>> BitAnd<T> for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn bitand(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Bitboard::bits(self.bits & rhs.bits)
    }
}

impl<T: Into<Bitboard>> BitAnd<T> for &Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn bitand(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Bitboard::bits(self.bits & rhs.bits)
    }
}

impl<T: Into<Bitboard>> BitAndAssign<T> for Bitboard {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.bits &= rhs.bits;
    }
}

impl<T: Into<Bitboard>> BitAndAssign<T> for &mut Bitboard {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.bits &= rhs.bits;
    }
}

impl<T: Into<Bitboard>> BitOr<T> for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn bitor(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Bitboard::bits(self.bits | rhs.bits)
    }
}

impl<T: Into<Bitboard>> BitOr<T> for &Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn bitor(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Bitboard::bits(self.bits | rhs.bits)
    }
}

impl<T: Into<Bitboard>> BitOrAssign<T> for Bitboard {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.bits |= rhs.bits;
    }
}

impl<T: Into<Bitboard>> BitOrAssign<T> for &mut Bitboard {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.bits |= rhs.bits;
    }
}

impl<T: Into<Bitboard>> BitXor<T> for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn bitxor(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Bitboard::bits(self.bits ^ rhs.bits)
    }
}

impl<T: Into<Bitboard>> BitXor<T> for &Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn bitxor(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Bitboard::bits(self.bits ^ rhs.bits)
    }
}

impl<T: Into<Bitboard>> BitXorAssign<T> for Bitboard {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.bits ^= rhs.bits;
    }
}

impl<T: Into<Bitboard>> BitXorAssign<T> for &mut Bitboard {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.bits ^= rhs.bits;
    }
}

impl Not for Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn not(self) -> Self::Output {
        Bitboard::bits(!self.bits)
    }
}

impl Not for &Bitboard {
    type Output = Bitboard;

    #[inline(always)]
    fn not(self) -> Self::Output {
        Bitboard::bits(!self.bits)
    }
}
