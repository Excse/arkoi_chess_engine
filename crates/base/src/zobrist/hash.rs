use std::{
    fmt::{Binary, LowerHex, Octal, UpperHex},
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign},
};

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
