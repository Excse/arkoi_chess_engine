pub mod error;
mod tests;

use std::{
    fmt::{Display, LowerHex, UpperHex},
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not},
    str::FromStr,
};

use crate::board::Board;

use self::error::{InvalidSquareFormat, SquareError};

#[derive(Debug, Default, Hash, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Square {
    pub rank: u8,
    pub file: u8,
    pub index: usize,
}

impl Square {
    pub fn new(rank: u8, file: u8) -> Self {
        let index = (rank * 8) + file;
        Self {
            rank,
            file,
            index: index as usize,
        }
    }

    pub fn index(index: usize) -> Self {
        let rank = (index / 8) as u8;
        let file = (index % 8) as u8;
        Self { rank, file, index }
    }

    fn in_board(&self) -> bool {
        let rank = self.rank as usize;
        let file = self.file as usize;

        let between_rank = rank >= Board::MIN_RANK && rank <= Board::MAX_RANK;
        let between_file = file >= Board::MIN_FILE && file <= Board::MAX_FILE;

        between_rank && between_file
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rank = self.rank + 1;
        let file = (b'a' + self.file) as char;

        write!(f, "{}{}", file, rank)
    }
}

impl FromStr for Square {
    type Err = SquareError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut stream = input.chars();

        let file = match stream.next() {
            Some(char) if char.is_ascii_lowercase() => char as u8 - b'a',
            _ => return Err(InvalidSquareFormat::new(input).into()),
        };
        let rank = match stream.next() {
            Some(char) if char.is_digit(10) => (char as u8 - b'0') - 1,
            _ => return Err(InvalidSquareFormat::new(input).into()),
        };

        let square = Self::new(rank, file);
        Ok(square)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bitboard {
    pub bits: u64,
}

impl Bitboard {
    pub const RANK_2: Bitboard = Bitboard::bits(0xFF00);
    pub const RANK_7: Bitboard = Bitboard::bits(0xFF000000000000);

    pub const fn bits(bits: u64) -> Self {
        Self { bits }
    }

    pub fn index(index: u8) -> Self {
        Self {
            bits: 1u64 << index,
        }
    }

    pub fn get_trailing_index(&self) -> usize {
        self.bits.trailing_zeros() as usize
    }

    pub fn get_leading_index(&self) -> usize {
        63 - self.bits.leading_zeros() as usize
    }

    pub fn is_set(&self, square: Square) -> bool {
        let new = self & square;
        new.bits != 0
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
    fn from(value: Square) -> Self {
        Bitboard::index(value.index as u8)
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

impl BitXorAssign<Square> for Bitboard {
    fn bitxor_assign(&mut self, rhs: Square) {
        let rhs: Bitboard = rhs.into();
        self.bits ^= rhs.bits
    }
}

impl BitAnd<u64> for Square {
    type Output = Bitboard;

    #[inline]
    fn bitand(self, rhs: u64) -> Self::Output {
        let lhs: Bitboard = self.into();
        let rhs: Bitboard = rhs.into();
        Bitboard::bits(lhs.bits & rhs.bits)
    }
}

impl BitAnd for Bitboard {
    type Output = Bitboard;

    #[inline]
    fn bitand(self, rhs: Bitboard) -> Self::Output {
        Bitboard::bits(self.bits & rhs.bits)
    }
}

impl BitAnd<Square> for Bitboard {
    type Output = Bitboard;

    #[inline]
    fn bitand(self, rhs: Square) -> Self::Output {
        let rhs: Bitboard = rhs.into();
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

impl BitAnd<&Bitboard> for Bitboard {
    type Output = Bitboard;

    #[inline]
    fn bitand(self, rhs: &Bitboard) -> Self::Output {
        Bitboard::bits(self.bits & rhs.bits)
    }
}

impl BitAnd<Square> for &Bitboard {
    type Output = Bitboard;

    #[inline]
    fn bitand(self, rhs: Square) -> Self::Output {
        let rhs: Bitboard = rhs.into();
        Bitboard::bits(self.bits & rhs.bits)
    }
}

impl BitAnd<&Bitboard> for u64 {
    type Output = Bitboard;

    #[inline]
    fn bitand(self, rhs: &Bitboard) -> Self::Output {
        Bitboard::bits(self & rhs.bits)
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

impl BitAnd<Bitboard> for u64 {
    type Output = Bitboard;

    #[inline]
    fn bitand(self, rhs: Bitboard) -> Self::Output {
        Bitboard::bits(self & rhs.bits)
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        self.bits &= rhs.bits
    }
}

impl BitAndAssign<u64> for Bitboard {
    fn bitand_assign(&mut self, rhs: u64) {
        self.bits &= rhs
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

impl BitOrAssign<u64> for Bitboard {
    fn bitor_assign(&mut self, rhs: u64) {
        self.bits |= rhs;
    }
}

impl BitOrAssign<Square> for Bitboard {
    fn bitor_assign(&mut self, rhs: Square) {
        let rhs: Bitboard = rhs.into();
        self.bits |= rhs.bits
    }
}

impl Not for &Bitboard {
    type Output = Bitboard;

    fn not(self) -> Self::Output {
        Bitboard::bits(!self.bits)
    }
}
