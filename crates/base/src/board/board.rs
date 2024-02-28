use std::fmt::{Display, Formatter};

use crate::{
    bitboard::Bitboard,
    r#move::Move,
    square::{constants::*, Square},
    zobrist::{ZobristHash, ZobristHasher},
};

use super::{
    color::Color,
    error::{
        BoardError, InvalidEnPassant, NotEnoughParts, WrongActiveColor, WrongCastlingAvailibility,
    },
    piece::{Tile, Piece},
};

#[derive(Debug, Clone)]
pub struct Board {
    bitboards: [[Bitboard; Piece::COUNT]; Color::COUNT],
    pieces: [Option<Tile>; Board::SIZE],
    white: Bitboard,
    black: Bitboard,
    occupied: Bitboard,
    hasher: ZobristHasher,
    midgame: [i32; Color::COUNT],
    endgame: [i32; Color::COUNT],
    gamephase: i32,
    gamestate: GameState,
    history: Vec<GameState>,
}

#[derive(Debug, Clone)]
pub struct EnPassant {
    pub to_move: Square,
    pub to_capture: Square,
}

impl EnPassant {
    pub const fn new(to_move: Square, to_capture: Square) -> Self {
        Self {
            to_move,
            to_capture,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GameState {
    hash: ZobristHash,
    active: Color,
    halfmoves: u16,
    fullmoves: u16,
    en_passant: Option<EnPassant>,
    white_kingside: bool,
    white_queenside: bool,
    black_kingside: bool,
    black_queenside: bool,
    pinned: Bitboard,
    checkers: Bitboard,
    attacked: Bitboard,
    captured_piece: Piece,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            hash: ZobristHash::default(),
            active: Color::White,
            black_kingside: false,
            white_kingside: false,
            black_queenside: false,
            white_queenside: false,
            en_passant: None,
            halfmoves: 0,
            fullmoves: 0,
            pinned: Bitboard::default(),
            checkers: Bitboard::default(),
            attacked: Bitboard::default(),
            captured_piece: Piece::None,
        }
    }
}

impl Board {
    pub const STARTPOS_FEN: &'static str =
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    pub const MAX_RANK: usize = 7;
    pub const MAX_FILE: usize = 7;
    pub const MIN_RANK: usize = 0;
    pub const MIN_FILE: usize = 0;
    pub const SIZE: usize = 64;

    pub fn default(hasher: ZobristHasher) -> Board {
        Board::from_str(Board::STARTPOS_FEN, hasher).unwrap()
    }

    pub fn empty(hasher: ZobristHasher) -> Board {
        Board {
            bitboards: [[Bitboard::default(); Piece::COUNT]; Color::COUNT],
            pieces: [None; Board::SIZE],
            occupied: Bitboard::default(),
            white: Bitboard::default(),
            black: Bitboard::default(),
            hasher,
            midgame: [0; Color::COUNT],
            endgame: [0; Color::COUNT],
            gamephase: 0,
            gamestate: GameState::default(),
            history: Vec::with_capacity(128),
        }
    }

    #[inline(always)]
    pub fn board_hash(&self) -> ZobristHash {
        self.hasher.hash(self)
    }

    #[inline(always)]
    pub fn swap_active(&mut self) {
        self.gamestate.active = self.gamestate.active.other();
        self.gamestate.hash ^= self.hasher.side_hash();
    }

    #[inline(always)]
    pub fn get_piece_count(&self, color: Color, piece: Piece) -> usize {
        let bitboard = self.get_piece_board(color, piece);
        bitboard.count_ones()
    }

    pub fn get_king_square(&self, color: Color) -> Square {
        let king_bb = self.get_piece_board(color, Piece::King);
        debug_assert_eq!(king_bb.count_ones(), 1);

        let index = king_bb.get_trailing_index();
        let square = Square::from_index(index);
        square
    }

    pub fn get_squares_by_piece(&self, color: Color, piece: Piece) -> Vec<Square> {
        let pieces = self.get_piece_board(color, piece);
        pieces.get_squares()
    }

    #[inline(always)]
    pub fn set_piece_type(&mut self, square: Square, piece: Option<Tile>) {
        self.pieces[usize::from(square)] = piece;
    }

    #[inline(always)]
    pub fn get_tile(&self, square: Square) -> Option<Tile> {
        self.pieces[usize::from(square)]
    }

    #[inline(always)]
    pub const fn get_piece_board(&self, color: Color, piece: Piece) -> Bitboard {
        self.bitboards[color.index()][piece.index()]
    }

    #[inline(always)]
    pub const fn get_occupied(&self, color: Color) -> Bitboard {
        match color {
            Color::White => self.white,
            Color::Black => self.black,
        }
    }

    #[inline(always)]
    pub const fn get_all_occupied(&self) -> Bitboard {
        self.occupied
    }

    pub fn update_game_state(&mut self) {
        let color = self.gamestate.active;

        let king_square = self.get_king_square(color);
        let all_occupied = self.get_all_occupied();

        let mut pinners = Bitboard::default();

        let attacked_blockers = all_occupied ^ king_square;
        let mut attacked = Bitboard::default();

        let queens = self.get_piece_board(color.other(), Piece::Queen);

        let bishops = self.get_piece_board(color.other(), Piece::Bishop);
        let bishop_attacks = king_square.get_bishop_attacks(Bitboard::EMPTY);
        let bishop_combined = bishops | queens;

        pinners ^= bishop_attacks & bishop_combined;
        for source in bishop_combined {
            attacked |= source.get_bishop_attacks(attacked_blockers);
        }

        let rooks = self.get_piece_board(color.other(), Piece::Rook);
        let rook_attacks = king_square.get_rook_attacks(Bitboard::EMPTY);
        let rook_combined = rooks | queens;

        pinners ^= rook_attacks & (rooks | queens);
        for source in rook_combined {
            attacked |= source.get_rook_attacks(attacked_blockers);
        }

        let mut checkers = Bitboard::default();
        let mut pinned = Bitboard::default();

        for pinner in pinners {
            let between = pinner.get_between(king_square) & all_occupied;
            if between.is_empty() {
                checkers ^= pinner;
            } else if between.count_ones() == 1 {
                pinned ^= between;
            }
        }

        let knights = self.get_piece_board(color.other(), Piece::Knight);
        let knight_moves = king_square.get_knight_moves();

        checkers ^= knight_moves & knights;
        for source in knights {
            attacked |= source.get_knight_moves();
        }

        let pawns = self.get_piece_board(color.other(), Piece::Pawn);
        let pawn_attacks = king_square.get_pawn_attacks(color);

        checkers ^= pawn_attacks & pawns;
        for source in pawns {
            attacked |= source.get_pawn_attacks(color.other());
        }

        let other_king = self.get_king_square(color.other());
        let king_moves = other_king.get_king_moves();
        attacked |= king_moves;

        self.gamestate.attacked = attacked;
        self.gamestate.checkers = checkers;
        self.gamestate.pinned = pinned;
    }

    pub fn toggle(&mut self, color: Color, piece: Piece, square: Square) {
        debug_assert!(piece != Piece::None);

        let color_index = color.index();
        let piece_index = piece.index();
        self.bitboards[color_index][piece_index] ^= square;

        let mut midgame_value = square.get_midgame_value(color, piece);
        midgame_value += piece.get_midgame_value();

        let mut endgame_value = square.get_endgame_value(color, piece);
        endgame_value += piece.get_endgame_value();

        let gamephase = piece.get_gamephase_value();

        if self.get_tile(square).is_some() {
            self.set_piece_type(square, None);

            self.midgame[color.index()] -= midgame_value;
            self.endgame[color.index()] -= endgame_value;
            self.gamephase -= gamephase;
        } else {
            self.set_piece_type(square, Some(Tile::new(piece, color)));

            self.midgame[color.index()] += midgame_value;
            self.endgame[color.index()] += endgame_value;
            self.gamephase += gamephase;
        }

        match color {
            Color::White => self.white ^= square,
            Color::Black => self.black ^= square,
        }

        self.occupied ^= square;

        self.gamestate.hash ^= self.hasher.piece_hash(piece, color, square);
    }

    pub fn remove_castle(&mut self, color: Color, short: bool) {
        match (color, short) {
            (Color::White, true) => {
                if self.gamestate.white_kingside {
                    self.gamestate.white_kingside = false;
                    self.gamestate.hash ^= self.hasher.castling[0];
                }
            }
            (Color::White, false) => {
                if self.gamestate.white_queenside {
                    self.gamestate.white_queenside = false;
                    self.gamestate.hash ^= self.hasher.castling[1];
                }
            }
            (Color::Black, true) => {
                if self.gamestate.black_kingside {
                    self.gamestate.black_kingside = false;
                    self.gamestate.hash ^= self.hasher.castling[2];
                }
            }
            (Color::Black, false) => {
                if self.gamestate.black_queenside {
                    self.gamestate.black_queenside = false;
                    self.gamestate.hash ^= self.hasher.castling[3];
                }
            }
        }
    }

    pub fn make(&mut self, mov: Move) {
        let gamestate = self.gamestate.clone();
        self.history.push(gamestate);

        let from = mov.from();
        let to = mov.to();

        let piece = match self.get_tile(from) {
            Some(tile) => tile.piece,
            None => panic!("No piece found at {}", from),
        };

        if mov.is_en_passant() {
            let to_capture = self.en_passant().clone().unwrap().to_capture;
            self.toggle(self.other(), Piece::Pawn, to_capture);
        }

        // Each turn reset the en passant square
        if let Some(en_passant) = &self.gamestate.en_passant {
            self.gamestate.hash ^= self.hasher.en_passant_hash(en_passant.to_capture);
            self.gamestate.en_passant = None;
        }

        if piece == Piece::Pawn {
            self.gamestate.halfmoves = 0;
        } else {
            self.gamestate.halfmoves += 1;
        }

        if self.gamestate.active == Color::Black {
            self.gamestate.fullmoves += 1;
        }

        if mov.is_double_pawn() {
            let to_move_index = i8::from(to) + self.gamestate.active.en_passant_offset();
            let to_move = Square::from_index(to_move_index as u8);
            self.gamestate.en_passant = Some(EnPassant::new(to_move, to));

            self.gamestate.hash ^= self.hasher.en_passant_hash(to_move);
        }

        if mov.is_capture() && !mov.is_en_passant() {
            let captured_piece = match self.get_tile(to) {
                Some(tile) => tile.piece,
                None => panic!("No piece found"),
            };
            self.toggle(self.gamestate.active.other(), captured_piece, to);
            self.gamestate.captured_piece = captured_piece;

            self.gamestate.halfmoves = 0;

            match (captured_piece, to) {
                (Piece::Rook, A1) => self.remove_castle(Color::White, false),
                (Piece::Rook, H1) => self.remove_castle(Color::White, true),
                (Piece::Rook, A8) => self.remove_castle(Color::Black, false),
                (Piece::Rook, H8) => self.remove_castle(Color::Black, true),
                _ => {}
            }
        }

        if !mov.is_promotion() {
            self.toggle(self.gamestate.active, piece, from);
            self.toggle(self.gamestate.active, piece, to);
        }

        match (piece, from) {
            (Piece::Rook, A1) => self.remove_castle(Color::White, false),
            (Piece::Rook, H1) => self.remove_castle(Color::White, true),
            (Piece::Rook, A8) => self.remove_castle(Color::Black, false),
            (Piece::Rook, H8) => self.remove_castle(Color::Black, true),
            (Piece::King, _) => {
                self.remove_castle(self.gamestate.active, false);
                self.remove_castle(self.gamestate.active, true);
            }

            _ => {}
        }

        if mov.is_castling() {
            match to {
                G1 => {
                    self.toggle(self.gamestate.active, Piece::Rook, H1);
                    self.toggle(self.gamestate.active, Piece::Rook, F1);
                }
                C1 => {
                    self.toggle(self.gamestate.active, Piece::Rook, A1);
                    self.toggle(self.gamestate.active, Piece::Rook, D1);
                }
                G8 => {
                    self.toggle(self.gamestate.active, Piece::Rook, H8);
                    self.toggle(self.gamestate.active, Piece::Rook, F8);
                }
                C8 => {
                    self.toggle(self.gamestate.active, Piece::Rook, A8);
                    self.toggle(self.gamestate.active, Piece::Rook, D8);
                }
                _ => panic!("Invalid castling move"),
            }
        } else if mov.is_promotion() {
            self.toggle(self.gamestate.active, piece, from);

            let promoted = mov.flag().get_promotion_piece();
            self.toggle(self.gamestate.active, promoted, to);
        }

        self.swap_active();

        self.update_game_state();
    }

    pub fn unmake(&mut self, mov: Move) {
        let from = mov.from();
        let to = mov.to();

        self.swap_active();

        let piece = match self.get_tile(to) {
            Some(tile) => tile.piece,
            None => panic!("No piece found at {}", to),
        };

        if mov.is_castling() {
            match to {
                G1 => {
                    self.toggle(self.gamestate.active, Piece::Rook, H1);
                    self.toggle(self.gamestate.active, Piece::Rook, F1);
                }
                C1 => {
                    self.toggle(self.gamestate.active, Piece::Rook, A1);
                    self.toggle(self.gamestate.active, Piece::Rook, D1);
                }
                G8 => {
                    self.toggle(self.gamestate.active, Piece::Rook, H8);
                    self.toggle(self.gamestate.active, Piece::Rook, F8);
                }
                C8 => {
                    self.toggle(self.gamestate.active, Piece::Rook, A8);
                    self.toggle(self.gamestate.active, Piece::Rook, D8);
                }
                _ => panic!("Invalid castling move"),
            }
        } else if mov.is_promotion() {
            self.toggle(self.gamestate.active, Piece::Pawn, from);

            let promoted = mov.flag().get_promotion_piece();
            self.toggle(self.gamestate.active, promoted, to);
        }

        if !mov.is_promotion() {
            self.toggle(self.gamestate.active, piece, from);
            self.toggle(self.gamestate.active, piece, to);
        }

        if mov.is_capture() && !mov.is_en_passant() {
            let captured_piece = self.gamestate.captured_piece;
            self.toggle(self.gamestate.active.other(), captured_piece, to);
        }

        let gamestate = self.history.pop();
        if let Some(gamestate) = gamestate {
            if mov.is_en_passant() {
                let to_capture = gamestate.en_passant.clone().unwrap().to_capture;
                self.toggle(self.other(), Piece::Pawn, to_capture);
            }

            self.gamestate = gamestate;
        }
    }

    pub fn make_moves(&mut self, input: &Vec<String>) -> Result<Vec<Move>, BoardError> {
        let mut moves = Vec::new();

        for mov in input {
            let mov = Move::parse(self, mov)?;
            moves.push(mov);

            self.make(mov);
        }

        Ok(moves)
    }

    pub fn make_null(&mut self) {
        let gamestate = self.gamestate.clone();
        self.history.push(gamestate);

        // Each turn reset the en passant square
        if let Some(en_passant) = &self.gamestate.en_passant {
            self.gamestate.hash ^= self.hasher.en_passant_hash(en_passant.to_capture);
            self.gamestate.en_passant = None;
        }

        if self.gamestate.active == Color::Black {
            self.gamestate.fullmoves += 1;
        }

        self.swap_active();

        self.update_game_state();
    }

    pub fn unmake_null(&mut self) {
        let game_state = self.history.pop();
        if let Some(game_state) = game_state {
            self.gamestate = game_state;
        }
    }

    // TODO: Optimize this, maybe with a hashmap
    pub fn is_threefold_repetition(&self) -> bool {
        let mut count = 0;

        for gamestate in self.history.iter().rev() {
            if gamestate.hash == self.gamestate.hash {
                count += 1;
            }

            if count == 3 {
                break;
            }
        }

        count == 3
    }

    pub fn to_fen(&self) -> String {
        let mut fen = String::new();
        for rank in (0..8).rev() {
            let mut empty = 0;

            for file in 0..8 {
                let square = Square::new(rank, file);
                let piece = self.get_tile(square);
                match piece {
                    Some(piece) => {
                        if empty > 0 {
                            fen.push_str(&empty.to_string());
                            empty = 0;
                        }

                        let piece = piece.to_fen();
                        fen.push(piece);
                    }
                    None => empty += 1,
                }
            }

            if empty > 0 {
                fen.push_str(&empty.to_string());
            }

            if rank > 0 {
                fen.push('/');
            }
        }

        fen.push(' ');
        let active = match self.gamestate.active {
            Color::White => "w",
            Color::Black => "b",
        };
        fen.push_str(active);

        fen.push(' ');
        let mut castling = String::new();
        if self.gamestate.white_kingside {
            castling.push('K');
        }
        if self.gamestate.white_queenside {
            castling.push('Q');
        }
        if self.gamestate.black_kingside {
            castling.push('k');
        }
        if self.gamestate.black_queenside {
            castling.push('q');
        }
        if castling.is_empty() {
            castling.push('-');
        }
        fen.push_str(&castling);

        fen.push(' ');
        let en_passant = match &self.gamestate.en_passant {
            Some(en_passant) => en_passant.to_capture.to_string(),
            None => "-".to_string(),
        };
        fen.push_str(&en_passant);

        fen.push(' ');
        let halfmoves = self.gamestate.halfmoves.to_string();
        fen.push_str(&halfmoves);

        fen.push(' ');
        let fullmoves = self.gamestate.fullmoves.to_string();
        fen.push_str(&fullmoves);

        fen
    }

    #[inline(always)]
    pub const fn active(&self) -> Color {
        self.gamestate.active
    }

    #[inline(always)]
    pub const fn other(&self) -> Color {
        self.active().other()
    }

    #[inline(always)]
    pub const fn hash(&self) -> ZobristHash {
        self.gamestate.hash
    }

    #[inline(always)]
    pub const fn hasher(&self) -> &ZobristHasher {
        &self.hasher
    }

    #[inline(always)]
    pub const fn halfmoves(&self) -> u16 {
        self.gamestate.halfmoves
    }

    #[inline(always)]
    pub const fn en_passant(&self) -> &Option<EnPassant> {
        &self.gamestate.en_passant
    }

    #[inline(always)]
    pub const fn can_white_kingside(&self) -> bool {
        self.gamestate.white_kingside
    }

    #[inline(always)]
    pub const fn can_white_queenside(&self) -> bool {
        self.gamestate.white_queenside
    }

    #[inline(always)]
    pub const fn can_black_kingside(&self) -> bool {
        self.gamestate.black_kingside
    }

    #[inline(always)]
    pub const fn can_black_queenside(&self) -> bool {
        self.gamestate.black_queenside
    }

    #[inline(always)]
    pub const fn midgame(&self, color: Color) -> i32 {
        self.midgame[color.index()]
    }

    #[inline(always)]
    pub const fn endgame(&self, color: Color) -> i32 {
        self.endgame[color.index()]
    }

    #[inline(always)]
    pub const fn gamephase(&self) -> i32 {
        self.gamephase
    }

    #[inline(always)]
    pub const fn checkers(&self) -> Bitboard {
        self.gamestate.checkers
    }

    #[inline(always)]
    pub fn is_check(&self) -> bool {
        !self.gamestate.checkers.is_empty()
    }

    #[inline(always)]
    pub const fn pinned(&self) -> Bitboard {
        self.gamestate.pinned
    }

    #[inline(always)]
    pub const fn attacked(&self) -> Bitboard {
        self.gamestate.attacked
    }

    #[inline(always)]
    pub const fn gamestate(&self) -> &GameState {
        &self.gamestate
    }
}

impl Display for Board {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(fmt, " +---+---+---+---+---+---+---+---+")?;

        for rank in (0..8).rev() {
            for file in 0..8 {
                let square = Square::new(rank, file);
                let piece = match self.get_tile(square) {
                    Some(piece) => piece.to_fen(),
                    None => ' ',
                };

                write!(fmt, " | {}", piece)?;
            }

            writeln!(fmt, " | {}", rank + 1)?;

            writeln!(fmt, " +---+---+---+---+---+---+---+---+")?;
        }

        write!(fmt, "   a   b   c   d   e   f   g   h")
    }
}

impl Board {
    pub fn from_str(fen: &str, hasher: ZobristHasher) -> Result<Self, BoardError> {
        let fen_parts: Vec<&str> = fen.split(" ").collect();
        if fen_parts.len() != 6 {
            return Err(NotEnoughParts.into());
        }

        let mut board = Board::empty(hasher);

        let ranks = fen_parts[0].split("/");
        for (rank_index, rank) in ranks.enumerate() {
            let mut file_index: u8 = 0;
            for piece in rank.chars() {
                if piece.is_digit(10) {
                    let digit = piece.to_digit(10).unwrap();
                    file_index += digit as u8;
                    continue;
                }

                let rank_index = 7 - rank_index as u8;

                let Tile { piece, color } = Tile::from_fen(piece)?;
                let square = Square::new(rank_index, file_index);
                board.toggle(color, piece, square);

                file_index += 1;
            }
        }

        let active_color = fen_parts[1];
        match active_color {
            "w" => board.gamestate.active = Color::White,
            "b" => board.gamestate.active = Color::Black,
            _ => return Err(WrongActiveColor::new(active_color).into()),
        }

        let castling_availibility = fen_parts[2];
        for availibility in castling_availibility.chars() {
            if availibility == '-' {
                break;
            }

            let piece = Tile::from_fen(availibility)?;
            match (piece.color, piece.piece) {
                (Color::Black, Piece::Queen) => board.gamestate.black_queenside = true,
                (Color::White, Piece::Queen) => board.gamestate.white_queenside = true,
                (Color::Black, Piece::King) => board.gamestate.black_kingside = true,
                (Color::White, Piece::King) => board.gamestate.white_kingside = true,
                _ => return Err(WrongCastlingAvailibility::new(availibility).into()),
            }
        }

        let en_passant = fen_parts[3];
        if en_passant != "-" {
            let file = en_passant
                .chars()
                .nth(0)
                .ok_or(InvalidEnPassant::new(en_passant))?;
            let file = file as u8 - b'a';

            let rank = en_passant
                .chars()
                .nth(1)
                .ok_or(InvalidEnPassant::new(en_passant))?;
            let rank = rank.to_digit(10).ok_or(InvalidEnPassant::new(en_passant))?;
            if rank < 1 || rank > 8 {
                return Err(InvalidEnPassant::new(en_passant).into());
            }

            let to_move_rank = rank as u8 - 1;
            let to_move = Square::new(to_move_rank, file);

            let to_capture_rank = match board.gamestate.active {
                Color::White => to_move_rank - 1,
                Color::Black => to_move_rank + 1,
            };
            let to_capture = Square::new(to_capture_rank, file);

            board.gamestate.en_passant = Some(EnPassant::new(to_move, to_capture));
        }

        let halfemoves = fen_parts[4].parse::<u16>()?;
        board.gamestate.halfmoves = halfemoves;

        let fullmoves = fen_parts[5].parse::<u16>()?;
        board.gamestate.fullmoves = fullmoves;

        let hash = board.board_hash();
        board.gamestate.hash = hash;

        board.update_game_state();

        Ok(board)
    }
}
