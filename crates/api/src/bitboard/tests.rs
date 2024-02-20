#[cfg(test)]
mod bitboard {
    use crate::{bitboard::Bitboard, square::constants::*};

    #[test]
    fn index() {
        let bitboard = Bitboard::from_index(10);
        assert_eq!(bitboard.get_leading_index(), 10);
        assert_eq!(bitboard.get_trailing_index(), 10);

        let bitboard = Bitboard::from_index(63);
        assert_eq!(bitboard.get_leading_index(), 63);
        assert_eq!(bitboard.get_trailing_index(), 63);

        let bitboard = Bitboard::from_index(0);
        assert_eq!(bitboard.get_leading_index(), 0);
        assert_eq!(bitboard.get_trailing_index(), 0);
    }

    #[test]
    fn is_set() {
        let bitboard = Bitboard::from_index(0);
        assert!(bitboard.is_set(A1));

        let bitboard = Bitboard::from_index(63);
        assert!(bitboard.is_set(H8));

        let bitboard = Bitboard::from_bits(0x1000000000);
        assert!(bitboard.is_set(E5));
    }
}
