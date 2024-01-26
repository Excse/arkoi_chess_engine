#[cfg(test)]
mod fen {
    use crate::board::{Board, Color, Piece};
    use std::str::FromStr;

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
}
