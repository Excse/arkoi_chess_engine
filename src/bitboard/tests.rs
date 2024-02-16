#[cfg(test)]
mod square {
    use std::str::FromStr;

    use crate::bitboard::{constants::*, square::Square};

    #[test]
    fn index() {
        assert_eq!(u8::from(A1), 0);
        assert_eq!(u8::from(H8), 63);
        assert_eq!(u8::from(A2), 8);
        assert_eq!(u8::from(D4), 27);
        assert_eq!(u8::from(F6), 45);
    }

    #[test]
    fn rank_file() {
        let square = A1;
        assert_eq!(square.rank(), 0);
        assert_eq!(square.file(), 0);

        let square = H8;
        assert_eq!(square.rank(), 7);
        assert_eq!(square.file(), 7);

        let square = A2;
        assert_eq!(square.rank(), 1);
        assert_eq!(square.file(), 0);

        let square = D4;
        assert_eq!(square.rank(), 3);
        assert_eq!(square.file(), 3);

        let square = F6;
        assert_eq!(square.rank(), 5);
        assert_eq!(square.file(), 5);
    }

    #[test]
    fn in_board() {
        assert!(A1.in_board());
        assert!(H8.in_board());
        assert!(A2.in_board());
        assert!(D4.in_board());
        assert!(E4.in_board());
    }

    #[test]
    fn from_str() {
        let square = Square::from_str("a1").unwrap();
        assert_eq!(square, A1);

        let square = Square::from_str("h8").unwrap();
        assert_eq!(square, H8);

        let square = Square::from_str("b1").unwrap();
        assert_eq!(square, B1);
    }

    #[test]
    fn display() {
        assert_eq!(A1.to_string(), "a1");
        assert_eq!(H8.to_string(), "h8");
        assert_eq!(B1.to_string(), "b1");
    }
}

#[cfg(test)]
mod bitboard {
    use crate::bitboard::{constants::*, Bitboard};

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
