#[cfg(test)]
mod color {
    use crate::board::Color;

    #[test]
    fn negated() {
        assert_eq!(!Color::White, Color::Black);
        assert_eq!(!Color::Black, Color::White);
    }

    #[test]
    fn index() {
        assert_eq!(Color::COUNT, 2);
        assert_eq!(Color::at(0), Some(Color::Black));
        assert_eq!(Color::at(1), Some(Color::White));
        assert_eq!(Color::at(2), None);
    }
}

#[cfg(test)]
mod piece {
    use crate::board::Piece;

    #[test]
    fn index() {
        assert_eq!(Piece::COUNT, 6);
        assert_eq!(Piece::at(0), Some(Piece::Pawn));
        assert_eq!(Piece::at(1), Some(Piece::Knight));
        assert_eq!(Piece::at(2), Some(Piece::Bishop));
        assert_eq!(Piece::at(3), Some(Piece::Rook));
        assert_eq!(Piece::at(4), Some(Piece::Queen));
        assert_eq!(Piece::at(5), Some(Piece::King));
        assert_eq!(Piece::at(6), None);
    }
}

#[cfg(test)]
mod colored_piece {
    use crate::board::{Color, ColoredPiece, Piece};

    #[test]
    fn from_fen() {
        let piece = ColoredPiece::from_fen('P').unwrap();
        assert_eq!(piece.color, Color::White);
        assert_eq!(piece.piece, Piece::Pawn);
        let piece = ColoredPiece::from_fen('p').unwrap();
        assert_eq!(piece.color, Color::Black);
        assert_eq!(piece.piece, Piece::Pawn);

        let piece = ColoredPiece::from_fen('N').unwrap();
        assert_eq!(piece.color, Color::White);
        assert_eq!(piece.piece, Piece::Knight);
        let piece = ColoredPiece::from_fen('n').unwrap();
        assert_eq!(piece.color, Color::Black);
        assert_eq!(piece.piece, Piece::Knight);

        let piece = ColoredPiece::from_fen('B').unwrap();
        assert_eq!(piece.color, Color::White);
        assert_eq!(piece.piece, Piece::Bishop);
        let piece = ColoredPiece::from_fen('b').unwrap();
        assert_eq!(piece.color, Color::Black);
        assert_eq!(piece.piece, Piece::Bishop);

        let piece = ColoredPiece::from_fen('R').unwrap();
        assert_eq!(piece.color, Color::White);
        assert_eq!(piece.piece, Piece::Rook);
        let piece = ColoredPiece::from_fen('r').unwrap();
        assert_eq!(piece.color, Color::Black);
        assert_eq!(piece.piece, Piece::Rook);

        let piece = ColoredPiece::from_fen('Q').unwrap();
        assert_eq!(piece.color, Color::White);
        assert_eq!(piece.piece, Piece::Queen);
        let piece = ColoredPiece::from_fen('q').unwrap();
        assert_eq!(piece.color, Color::Black);
        assert_eq!(piece.piece, Piece::Queen);

        let piece = ColoredPiece::from_fen('K').unwrap();
        assert_eq!(piece.color, Color::White);
        assert_eq!(piece.piece, Piece::King);
        let piece = ColoredPiece::from_fen('k').unwrap();
        assert_eq!(piece.color, Color::Black);
        assert_eq!(piece.piece, Piece::King);
    }

    #[test]
    fn to_fen() {
        let piece = ColoredPiece::new(Piece::Pawn, Color::White);
        assert_eq!(piece.to_fen(), 'P');
        let piece = ColoredPiece::new(Piece::Pawn, Color::Black);
        assert_eq!(piece.to_fen(), 'p');

        let piece = ColoredPiece::new(Piece::Knight, Color::White);
        assert_eq!(piece.to_fen(), 'N');
        let piece = ColoredPiece::new(Piece::Knight, Color::Black);
        assert_eq!(piece.to_fen(), 'n');

        let piece = ColoredPiece::new(Piece::Bishop, Color::White);
        assert_eq!(piece.to_fen(), 'B');
        let piece = ColoredPiece::new(Piece::Bishop, Color::Black);
        assert_eq!(piece.to_fen(), 'b');

        let piece = ColoredPiece::new(Piece::Rook, Color::White);
        assert_eq!(piece.to_fen(), 'R');
        let piece = ColoredPiece::new(Piece::Rook, Color::Black);
        assert_eq!(piece.to_fen(), 'r');

        let piece = ColoredPiece::new(Piece::Queen, Color::White);
        assert_eq!(piece.to_fen(), 'Q');
        let piece = ColoredPiece::new(Piece::Queen, Color::Black);
        assert_eq!(piece.to_fen(), 'q');

        let piece = ColoredPiece::new(Piece::King, Color::White);
        assert_eq!(piece.to_fen(), 'K');
        let piece = ColoredPiece::new(Piece::King, Color::Black);
        assert_eq!(piece.to_fen(), 'k');
    }
}

#[cfg(test)]
mod fen {
    use crate::{
        board::{Board, Color, Piece},
        move_generator::mov::Move,
    };
    use std::str::FromStr;

    #[test]
    fn swap_active() {
        let mut board = Board::default();
        assert_eq!(board.active, Color::White);

        board.swap_active();
        assert_eq!(board.active, Color::Black);

        board.swap_active();
        assert_eq!(board.active, Color::White);
    }

    #[test]
    fn fen_starting_position() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let board = Board::from_str(fen).unwrap();

        let king_bb = board.get_piece_board(Color::White, Piece::King);
        assert_eq!(king_bb.bits, 0x10);
        let king_bb = board.get_piece_board(Color::Black, Piece::King);
        assert_eq!(king_bb.bits, 0x1000000000000000);

        let pawn_bb = board.get_piece_board(Color::White, Piece::Pawn);
        assert_eq!(pawn_bb.bits, 0xff00);
        let pawn_bb = board.get_piece_board(Color::Black, Piece::Pawn);
        assert_eq!(pawn_bb.bits, 0xff000000000000);

        let knight_bb = board.get_piece_board(Color::White, Piece::Knight);
        assert_eq!(knight_bb.bits, 0x42);
        let knight_bb = board.get_piece_board(Color::Black, Piece::Knight);
        assert_eq!(knight_bb.bits, 0x4200000000000000);

        let bishop_bb = board.get_piece_board(Color::White, Piece::Bishop);
        assert_eq!(bishop_bb.bits, 0x24);
        let bishop_bb = board.get_piece_board(Color::Black, Piece::Bishop);
        assert_eq!(bishop_bb.bits, 0x2400000000000000);

        let rook_bb = board.get_piece_board(Color::White, Piece::Rook);
        assert_eq!(rook_bb.bits, 0x81);
        let rook_bb = board.get_piece_board(Color::Black, Piece::Rook);
        assert_eq!(rook_bb.bits, 0x8100000000000000);

        let queen_bb = board.get_piece_board(Color::White, Piece::Queen);
        assert_eq!(queen_bb.bits, 0x8);
        let queen_bb = board.get_piece_board(Color::Black, Piece::Queen);
        assert_eq!(queen_bb.bits, 0x800000000000000);
    }

    #[test]
    fn fen_custom_1() {
        let fen = "rnbq1bnr/pppk1ppp/8/1B1pp3/3PP3/5P2/PPP3PP/RNBQK1NR b KQ - 2 4";
        let board = Board::from_str(fen).unwrap();

        let king_bb = board.get_piece_board(Color::White, Piece::King);
        assert_eq!(king_bb.bits, 0x10);
        let king_bb = board.get_piece_board(Color::Black, Piece::King);
        assert_eq!(king_bb.bits, 0x8000000000000);

        let pawn_bb = board.get_piece_board(Color::White, Piece::Pawn);
        assert_eq!(pawn_bb.bits, 0x1820c700);
        let pawn_bb = board.get_piece_board(Color::Black, Piece::Pawn);
        assert_eq!(pawn_bb.bits, 0xe7001800000000);

        let knight_bb = board.get_piece_board(Color::White, Piece::Knight);
        assert_eq!(knight_bb.bits, 0x42);
        let knight_bb = board.get_piece_board(Color::Black, Piece::Knight);
        assert_eq!(knight_bb.bits, 0x4200000000000000);

        let bishop_bb = board.get_piece_board(Color::White, Piece::Bishop);
        assert_eq!(bishop_bb.bits, 0x200000004);
        let bishop_bb = board.get_piece_board(Color::Black, Piece::Bishop);
        assert_eq!(bishop_bb.bits, 0x2400000000000000);

        let rook_bb = board.get_piece_board(Color::White, Piece::Rook);
        assert_eq!(rook_bb.bits, 0x81);
        let rook_bb = board.get_piece_board(Color::Black, Piece::Rook);
        assert_eq!(rook_bb.bits, 0x8100000000000000);

        let queen_bb = board.get_piece_board(Color::White, Piece::Queen);
        assert_eq!(queen_bb.bits, 0x8);
        let queen_bb = board.get_piece_board(Color::Black, Piece::Queen);
        assert_eq!(queen_bb.bits, 0x800000000000000);
    }

    #[test]
    fn fen_custom_2() {
        let fen = "2q1kb2/1P1ppp1r/Q6p/PB4pn/4PP2/8/P5PP/RNB1K1NR b KQ - 0 16";
        let board = Board::from_str(fen).unwrap();

        let king_bb = board.get_piece_board(Color::White, Piece::King);
        assert_eq!(king_bb.bits, 0x10);
        let king_bb = board.get_piece_board(Color::Black, Piece::King);
        assert_eq!(king_bb.bits, 0x1000000000000000);

        let pawn_bb = board.get_piece_board(Color::White, Piece::Pawn);
        assert_eq!(pawn_bb.bits, 0x200013000c100);
        let pawn_bb = board.get_piece_board(Color::Black, Piece::Pawn);
        assert_eq!(pawn_bb.bits, 0x38804000000000);

        let knight_bb = board.get_piece_board(Color::White, Piece::Knight);
        assert_eq!(knight_bb.bits, 0x42);
        let knight_bb = board.get_piece_board(Color::Black, Piece::Knight);
        assert_eq!(knight_bb.bits, 0x8000000000);

        let bishop_bb = board.get_piece_board(Color::White, Piece::Bishop);
        assert_eq!(bishop_bb.bits, 0x200000004);
        let bishop_bb = board.get_piece_board(Color::Black, Piece::Bishop);
        assert_eq!(bishop_bb.bits, 0x2000000000000000);

        let rook_bb = board.get_piece_board(Color::White, Piece::Rook);
        assert_eq!(rook_bb.bits, 0x81);
        let rook_bb = board.get_piece_board(Color::Black, Piece::Rook);
        assert_eq!(rook_bb.bits, 0x80000000000000);

        let queen_bb = board.get_piece_board(Color::White, Piece::Queen);
        assert_eq!(queen_bb.bits, 0x10000000000);
        let queen_bb = board.get_piece_board(Color::Black, Piece::Queen);
        assert_eq!(queen_bb.bits, 0x400000000000000);
    }

    #[test]
    fn fen_custom_3() {
        let fen = "rn2kbnr/pp5p/B1p5/1P2P3/3p2p1/6P1/PBPPb3/RN2K1q1 w Qkq - 0 17";
        let board = Board::from_str(fen).unwrap();

        let king_bb = board.get_piece_board(Color::White, Piece::King);
        assert_eq!(king_bb.bits, 0x10);
        let king_bb = board.get_piece_board(Color::Black, Piece::King);
        assert_eq!(king_bb.bits, 0x1000000000000000);

        let pawn_bb = board.get_piece_board(Color::White, Piece::Pawn);
        assert_eq!(pawn_bb.bits, 0x1200400d00);
        let pawn_bb = board.get_piece_board(Color::Black, Piece::Pawn);
        assert_eq!(pawn_bb.bits, 0x83040048000000);

        let knight_bb = board.get_piece_board(Color::White, Piece::Knight);
        assert_eq!(knight_bb.bits, 0x2);
        let knight_bb = board.get_piece_board(Color::Black, Piece::Knight);
        assert_eq!(knight_bb.bits, 0x4200000000000000);

        let bishop_bb = board.get_piece_board(Color::White, Piece::Bishop);
        assert_eq!(bishop_bb.bits, 0x10000000200);
        let bishop_bb = board.get_piece_board(Color::Black, Piece::Bishop);
        assert_eq!(bishop_bb.bits, 0x2000000000001000);

        let rook_bb = board.get_piece_board(Color::White, Piece::Rook);
        assert_eq!(rook_bb.bits, 0x1);
        let rook_bb = board.get_piece_board(Color::Black, Piece::Rook);
        assert_eq!(rook_bb.bits, 0x8100000000000000);

        let queen_bb = board.get_piece_board(Color::White, Piece::Queen);
        assert_eq!(queen_bb.bits, 0x0);
        let queen_bb = board.get_piece_board(Color::Black, Piece::Queen);
        assert_eq!(queen_bb.bits, 0x40);
    }

    #[test]
    fn fen_custom_4() {
        // TODO: Test FEN: r1bq1b1r/1pp2p1p/pk2pn2/3P2B1/N3P1p1/5N2/PPP2PPP/R2QK2R b KQ - 2 10
        let moves = "d2d4 g7g5 e2e4 g5g4 g1f3 a7a6 b1c3 b8c6 f1b5 e7e6 b5c6 g8f6 c6d7 e8d7 c1g5 d7c6 d4d5 c6b6 c3a4";
        let mut board = Board::default();
        for mov in moves.split(" ") {
            let mov = Move::parse(mov.to_string(), board.active, &board).unwrap();
            board.play(board.active, &mov).unwrap();
            board.swap_active();
        }

        let king_bb = board.get_piece_board(Color::White, Piece::King);
        assert_eq!(king_bb.bits, 0x10);
        let king_bb = board.get_piece_board(Color::Black, Piece::King);
        assert_eq!(king_bb.bits, 0x20000000000);

        let pawn_bb = board.get_piece_board(Color::White, Piece::Pawn);
        assert_eq!(pawn_bb.bits, 0x81000e700);
        let pawn_bb = board.get_piece_board(Color::Black, Piece::Pawn);
        assert_eq!(pawn_bb.bits, 0xa6110040000000);

        let knight_bb = board.get_piece_board(Color::White, Piece::Knight);
        assert_eq!(knight_bb.bits, 0x1200000);
        let knight_bb = board.get_piece_board(Color::Black, Piece::Knight);
        assert_eq!(knight_bb.bits, 0x200000000000);

        let bishop_bb = board.get_piece_board(Color::White, Piece::Bishop);
        assert_eq!(bishop_bb.bits, 0x4000000000);
        let bishop_bb = board.get_piece_board(Color::Black, Piece::Bishop);
        assert_eq!(bishop_bb.bits, 0x2400000000000000);

        let rook_bb = board.get_piece_board(Color::White, Piece::Rook);
        assert_eq!(rook_bb.bits, 0x81);
        let rook_bb = board.get_piece_board(Color::Black, Piece::Rook);
        assert_eq!(rook_bb.bits, 0x8100000000000000);

        let queen_bb = board.get_piece_board(Color::White, Piece::Queen);
        assert_eq!(queen_bb.bits, 0x8);
        let queen_bb = board.get_piece_board(Color::Black, Piece::Queen);
        assert_eq!(queen_bb.bits, 0x800000000000000);
    }
}
