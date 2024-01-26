#[cfg(test)]
mod square {
    use std::str::FromStr;

    use crate::bitboard::Square;

    #[test]
    fn index() {
        let square = Square::new(0, 0);
        assert_eq!(square.index, 0);

        let square = Square::new(7, 7);
        assert_eq!(square.index, 63);

        let square = Square::new(1, 0);
        assert_eq!(square.index, 8);

        let square = Square::new(3, 3);
        assert_eq!(square.index, 27);

        let square = Square::new(5, 5);
        assert_eq!(square.index, 45);
    }

    #[test]
    fn rank_file() {
        let square = Square::index(0);
        assert_eq!(square.rank, 0);
        assert_eq!(square.file, 0);

        let square = Square::index(63);
        assert_eq!(square.rank, 7);
        assert_eq!(square.file, 7);

        let square = Square::index(8);
        assert_eq!(square.rank, 1);
        assert_eq!(square.file, 0);

        let square = Square::index(27);
        assert_eq!(square.rank, 3);
        assert_eq!(square.file, 3);

        let square = Square::index(45);
        assert_eq!(square.rank, 5);
        assert_eq!(square.file, 5);
    }

    #[test]
    fn in_board() {
        let square = Square::new(0, 0);
        assert!(square.in_board());

        let square = Square::new(7, 7);
        assert!(square.in_board());

        let square = Square::new(1, 0);
        assert!(square.in_board());

        let square = Square::new(3, 3);
        assert!(square.in_board());

        let square = Square::new(5, 5);
        assert!(square.in_board());

        let square = Square::new(8, 0);
        assert!(!square.in_board());

        let square = Square::new(0, 8);
        assert!(!square.in_board());

        let square = Square::new(8, 8);
        assert!(!square.in_board());

        let square = Square::new(4, 20);
        assert!(!square.in_board());
    }

    #[test]
    fn from_str() {
        let square = Square::from_str("a1").unwrap();
        assert_eq!(square.rank, 0);
        assert_eq!(square.file, 0);

        let square = Square::from_str("h8").unwrap();
        assert_eq!(square.rank, 7);
        assert_eq!(square.file, 7);

        let square = Square::from_str("b1").unwrap();
        assert_eq!(square.rank, 0);
        assert_eq!(square.file, 1);
    }

    #[test]
    fn display() {
        let square = Square::new(0, 0);
        assert_eq!(square.to_string(), "a1");

        let square = Square::new(7, 7);
        assert_eq!(square.to_string(), "h8");

        let square = Square::new(0, 1);
        assert_eq!(square.to_string(), "b1");
    }
}

#[cfg(test)]
mod bitboard {
    use crate::bitboard::{Bitboard, Square};

    #[test]
    fn index() {
        let bitboard = Bitboard::index(10);
        assert_eq!(bitboard.get_leading_index(), 10);
        assert_eq!(bitboard.get_trailing_index(), 10);

        let bitboard = Bitboard::index(63);
        assert_eq!(bitboard.get_leading_index(), 63);
        assert_eq!(bitboard.get_trailing_index(), 63);

        let bitboard = Bitboard::index(0);
        assert_eq!(bitboard.get_leading_index(), 0);
        assert_eq!(bitboard.get_trailing_index(), 0);
    }

    #[test]
    fn is_set() {
        let bitboard = Bitboard::index(0);
        let square = Square::index(0);
        assert!(bitboard.is_set(square));

        let bitboard = Bitboard::index(63);
        let square = Square::index(63);
        assert!(bitboard.is_set(square));

        let bitboard = Bitboard::bits(0x1000000000);
        let square = Square::index(36);
        assert!(bitboard.is_set(square));
    }
}
