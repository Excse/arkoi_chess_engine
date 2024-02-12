#[cfg(test)]
mod color {
    use crate::board::Color;

    #[test]
    fn negated() {
        assert_eq!(!Color::White, Color::Black);
        assert_eq!(!Color::Black, Color::White);
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
    use rand::{rngs::StdRng, SeedableRng};

    use crate::{
        board::{zobrist::ZobristHasher, Board, Color, Piece},
        move_generator::mov::Move,
    };

    #[test]
    fn swap_active() {
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let mut board = Board::default(&hasher);
        assert_eq!(board.gamestate.active, Color::White);

        board.swap_active();
        assert_eq!(board.gamestate.active, Color::Black);

        board.swap_active();
        assert_eq!(board.gamestate.active, Color::White);
    }

    #[test]
    fn fen_starting_position() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let board = Board::from_str(fen, &hasher).unwrap();

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
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let board = Board::from_str(fen, &hasher).unwrap();

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
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let board = Board::from_str(fen, &hasher).unwrap();

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
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let board = Board::from_str(fen, &hasher).unwrap();

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
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let mut board = Board::default(&hasher);

        let moves = "d2d4 g7g5 e2e4 g5g4 g1f3 a7a6 b1c3 b8c6 f1b5 e7e6 b5c6 g8f6 c6d7 e8d7 c1g5 d7c6 d4d5 c6b6 c3a4";
        for mov in moves.split(" ") {
            let mov = Move::parse(mov.to_string(), &board).unwrap();
            board.make(&mov);
        }

        let fen = "r1bq1b1r/1pp2p1p/pk2pn2/3P2B1/N3P1p1/5N2/PPP2PPP/R2QK2R b KQ - 2 10";
        assert_eq!(board.to_fen(), fen);

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

#[cfg(test)]
mod zobrist {
    use rand::{rngs::StdRng, SeedableRng};

    use crate::{
        board::{zobrist::ZobristHasher, Board},
        move_generator::mov::Move,
    };

    #[test]
    fn check_startpos() {
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);
        let mut board = Board::default(&hasher);

        let moves = "d2d4 d7d5 e2e4 e7e5 c2c3 e5d4 f1b5 e8e7 g1f3 d5e4 f3d4 e4e3 e1g1 e3f2 f1f2";
        for mov in moves.split(" ") {
            let mov = Move::parse(mov.to_string(), &board).unwrap();
            board.make(&mov);

            assert_eq!(board.gamestate.hash, board.board_hash());
        }

        let fen = "rnbq1bnr/ppp1kppp/8/1B6/3N4/2P5/PP3RPP/RNBQ2K1 b - - 0 8";
        assert_eq!(board.to_fen(), fen);
    }

    #[test]
    fn not_same_hash_1() {
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);

        let first = "rnbqkbnr/1p1ppppp/8/P1p5/8/P7/2PPPPPP/RNBQKBNR b KQkq - 0 3";
        let board = Board::from_str(first, &hasher).unwrap();
        let first_hash = board.gamestate.hash;

        let second = "rnbqkbnr/1p1ppppp/8/p1P5/8/P7/2PPPPPP/RNBQKBNR b KQkq - 0 3";
        let board = Board::from_str(second, &hasher).unwrap();
        let second_hash = board.gamestate.hash;

        assert_ne!(first_hash, second_hash);
    }

    #[test]
    fn not_same_hash_2() {
        let mut rand = StdRng::seed_from_u64(42);
        let hasher = ZobristHasher::new(&mut rand);

        let first = "rn1qkbnr/ppp1pppp/8/3p4/8/PP5N/2PPPPPP/RNBQKB1R b KQkq - 2 3";
        let board = Board::from_str(first, &hasher).unwrap();
        let first_hash = board.gamestate.hash;

        let second = "rn1qkbnr/ppp1pppp/8/3p4/8/PP5b/2PPPPPP/RNBQKB1R b KQkq - 0 3";
        let board = Board::from_str(second, &hasher).unwrap();
        let second_hash = board.gamestate.hash;

        assert_ne!(first_hash, second_hash);
    }
}
