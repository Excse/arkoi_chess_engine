pub mod color;
pub mod error;
pub mod piece;
mod tests;
pub mod zobrist;

use std::fmt::Display;

use colored::Colorize;

use crate::{
    bitboard::{square::Square, squares::*, Bitboard},
    move_generator::mov::{EnPassant, Move, MoveKind},
};

use self::{
    color::Color,
    error::{
        BoardError, InvalidEnPassant, MultipleKings, NotEnoughParts, PieceNotFound,
        WrongActiveColor, WrongCastlingAvailibility,
    },
    piece::{ColoredPiece, Piece},
    zobrist::{ZobristHash, ZobristHasher},
};

#[derive(Debug, Clone, Copy)]
pub struct Board<'a> {
    pub bitboards: [[Bitboard; Piece::COUNT]; Color::COUNT],
    pub white: Bitboard,
    pub black: Bitboard,
    pub occupied: Bitboard,
    pub active: Color,
    pub black_kingside: bool,
    pub white_kingside: bool,
    pub black_queenside: bool,
    pub white_queenside: bool,
    pub en_passant: Option<EnPassant>,
    pub halfmoves: u16,
    pub fullmoves: u16,
    pub hasher: &'a ZobristHasher,
    pub hash: ZobristHash,
}

impl<'a> Board<'a> {
    pub const STARTPOS_FEN: &'static str =
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    pub const MAX_RANK: usize = 7;
    pub const MAX_FILE: usize = 7;
    pub const MIN_RANK: usize = 0;
    pub const MIN_FILE: usize = 0;
    pub const SIZE: usize = 64;

    pub fn default(hasher: &'a ZobristHasher) -> Board<'a> {
        Board::from_str(Board::STARTPOS_FEN, hasher).unwrap()
    }

    pub fn empty(hasher: &'a ZobristHasher) -> Board<'a> {
        Board {
            bitboards: [[Bitboard::default(); Piece::COUNT]; Color::COUNT],
            occupied: Bitboard::default(),
            white: Bitboard::default(),
            black: Bitboard::default(),
            active: Color::White,
            black_kingside: false,
            white_kingside: false,
            black_queenside: false,
            white_queenside: false,
            en_passant: None,
            halfmoves: 0,
            fullmoves: 0,
            hasher,
            hash: ZobristHash::default(),
        }
    }

    pub fn board_hash(&self) -> ZobristHash {
        let hash = self.hasher.hash(self);
        hash
    }

    pub fn swap_active(&mut self) {
        self.active = !self.active;
        self.hash ^= self.hasher.side;
    }

    pub fn get_king_square(&self, color: Color) -> Result<Option<Square>, BoardError> {
        let kings = self.get_squares_by_piece(color, Piece::King);

        match kings.len() {
            0 => Ok(None),
            1 => Ok(Some(kings[0])),
            _ => Err(MultipleKings::new(kings.len()).into()),
        }
    }

    pub fn get_squares_by_piece(&self, color: Color, piece: Piece) -> Vec<Square> {
        let mut squares = Vec::new();

        let mut pieces = *self.get_piece_board(color, piece);

        while pieces.bits != 0 {
            let index = pieces.bits.trailing_zeros() as usize;
            let from = Square::index(index);
            pieces ^= from;

            squares.push(from);
        }

        squares
    }

    pub fn get_piece_type(&self, color: Color, square: Square) -> Option<Piece> {
        let bitboards = &self.bitboards[color.index()];
        for (index, &piece_bb) in bitboards.iter().enumerate() {
            let contains_bb = piece_bb & square;
            if contains_bb.bits != 0 {
                let piece = Piece::at(index)?;
                return Some(piece);
            }
        }

        None
    }

    pub fn get_colored_piece_type(&self, square: Square) -> Option<ColoredPiece> {
        for (color_index, &color_bb) in self.bitboards.iter().enumerate() {
            for (piece_index, &piece_bb) in color_bb.iter().enumerate() {
                let contains_bb = piece_bb & square;
                if contains_bb.bits != 0 {
                    let piece = Piece::at(piece_index)?;
                    let color = Color::at(color_index)?;
                    let piece = ColoredPiece::new(piece, color);
                    return Some(piece);
                }
            }
        }

        None
    }

    pub fn get_piece_board(&self, color: Color, piece: Piece) -> &Bitboard {
        let index = color.index();
        let bitboards = &self.bitboards[index];

        let index = piece.index();
        &bitboards[index]
    }

    pub fn get_occupied(&self, color: Color) -> &Bitboard {
        match color {
            Color::White => &self.white,
            Color::Black => &self.black,
        }
    }

    #[inline]
    pub fn get_all_occupied(&self) -> &Bitboard {
        &self.occupied
    }

    pub fn toggle(&mut self, color: Color, piece: Piece, square: Square) {
        let color_index = color.index();
        let piece_index = piece.index();
        self.bitboards[color_index][piece_index] ^= square;

        match color {
            Color::White => self.white ^= square,
            Color::Black => self.black ^= square,
        }

        self.occupied ^= square;

        self.hash ^= self.hasher.get_piece_hash(piece, color, square);
    }

    pub fn remove_castle(&mut self, color: Color, short: bool) {
        match (color, short) {
            (Color::White, true) => {
                if self.white_kingside {
                    self.white_kingside = false;
                    self.hash ^= self.hasher.castling[0];
                }
            }
            (Color::White, false) => {
                if self.white_queenside {
                    self.white_queenside = false;
                    self.hash ^= self.hasher.castling[1];
                }
            }
            (Color::Black, true) => {
                if self.black_kingside {
                    self.black_kingside = false;
                    self.hash ^= self.hasher.castling[2];
                }
            }
            (Color::Black, false) => {
                if self.black_queenside {
                    self.black_queenside = false;
                    self.hash ^= self.hasher.castling[3];
                }
            }
        }
    }

    pub fn make(&mut self, mov: &Move) -> Result<(), BoardError> {
        // Each turn reset the en passant square
        if let Some(en_passant) = self.en_passant {
            let file_index = en_passant.to_capture.file() as usize;
            self.hash ^= self.hasher.en_passant[file_index];
            self.en_passant = None;
        }

        if mov.piece == Piece::Pawn {
            self.halfmoves = 0;
        } else {
            self.halfmoves += 1;
        }

        if self.active == Color::Black {
            self.fullmoves += 1;
        }

        self.en_passant = mov.is_en_passant();
        if let Some(en_passant) = self.en_passant {
            let file_index = en_passant.to_capture.file() as usize;
            self.hash ^= self.hasher.en_passant[file_index];
        }

        if !matches!(mov.kind, MoveKind::Promotion(_)) {
            self.toggle(self.active, mov.piece, mov.from);
            self.toggle(self.active, mov.piece, mov.to);
        }

        match (mov.piece, mov.from) {
            (Piece::Rook, A1) => self.remove_castle(Color::White, false),
            (Piece::Rook, H1) => self.remove_castle(Color::White, true),
            (Piece::Rook, A8) => self.remove_castle(Color::Black, false),
            (Piece::Rook, H8) => self.remove_castle(Color::Black, true),
            (Piece::King, _) => {
                self.remove_castle(self.active, false);
                self.remove_castle(self.active, true);
            }

            _ => {}
        }

        match mov.kind {
            MoveKind::Castle(ref castle) => {
                self.toggle(self.active, Piece::Rook, castle.rook_from);
                self.toggle(self.active, Piece::Rook, castle.rook_to);
            }
            MoveKind::EnPassant(ref en_passant) => {
                self.toggle(!self.active, Piece::Pawn, en_passant.capture);
            }
            MoveKind::Promotion(ref promotion) => {
                self.toggle(self.active, mov.piece, mov.from);
                self.toggle(self.active, promotion.promotion, mov.to);
            }
            MoveKind::Attack | MoveKind::Normal => {}
        }

        if mov.is_direct_attack() {
            let piece = self
                .get_piece_type(!self.active, mov.to)
                .ok_or(PieceNotFound)?;
            self.toggle(!self.active, piece, mov.to);

            if piece == Piece::Pawn {
                self.halfmoves = 0;
            }

            match (piece, mov.to) {
                (Piece::Rook, A1) => self.remove_castle(Color::White, false),
                (Piece::Rook, H1) => self.remove_castle(Color::White, true),
                (Piece::Rook, A8) => self.remove_castle(Color::Black, false),
                (Piece::Rook, H8) => self.remove_castle(Color::Black, true),
                _ => {}
            }
        }

        self.swap_active();
        Ok(())
    }

    pub fn to_fen(&self) -> String {
        let mut fen = String::new();
        for rank in (0..8).rev() {
            let mut empty = 0;

            for file in 0..8 {
                let square = Square::new(rank, file);
                let piece = self.get_colored_piece_type(square);
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
        let active = match self.active {
            Color::White => "w",
            Color::Black => "b",
        };
        fen.push_str(active);

        fen.push(' ');
        let mut castling = String::new();
        if self.white_kingside {
            castling.push('K');
        }
        if self.white_queenside {
            castling.push('Q');
        }
        if self.black_kingside {
            castling.push('k');
        }
        if self.black_queenside {
            castling.push('q');
        }
        if castling.is_empty() {
            castling.push('-');
        }
        fen.push_str(&castling);

        fen.push(' ');
        let en_passant = match self.en_passant {
            Some(en_passant) => en_passant.to_capture.to_string(),
            None => "-".to_string(),
        };
        fen.push_str(&en_passant);

        fen.push(' ');
        let halfmoves = self.halfmoves.to_string();
        fen.push_str(&halfmoves);

        fen.push(' ');
        let fullmoves = self.fullmoves.to_string();
        fen.push_str(&fullmoves);

        fen
    }
}

impl<'a> Display for Board<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for rank in (0..8).rev() {
            for size in 0..3 {
                if size == 1 {
                    write!(f, "{} ", rank + 1)?;
                } else {
                    write!(f, "  ")?;
                }

                for file in 0..8 {
                    let index = (8 * rank) + file;
                    let color = (index + rank) % 2;

                    let square = Square::new(rank, file);
                    let piece = self.get_colored_piece_type(square);
                    let piece = match (size, piece) {
                        (1, Some(piece)) => piece.to_fen(),
                        (_, _) => ' ',
                    };
                    let piece = format!("   {}   ", piece);

                    if color == 0 {
                        write!(f, "{}", piece.white().on_black())?;
                    } else {
                        write!(f, "{}", piece.black().on_white())?;
                    }
                }
                writeln!(f)?;
            }
        }

        writeln!(f, "     a      b      c      d      e      f      g      h")
    }
}

impl<'a> Board<'a> {
    pub fn from_str(fen: &str, hasher: &'a ZobristHasher) -> Result<Self, BoardError> {
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

                let ColoredPiece { piece, color } = ColoredPiece::from_fen(piece)?;
                let square = Square::new(rank_index, file_index);
                board.toggle(color, piece, square);

                file_index += 1;
            }
        }

        let active_color = fen_parts[1];
        match active_color {
            "w" => board.active = Color::White,
            "b" => board.active = Color::Black,
            _ => return Err(WrongActiveColor::new(active_color).into()),
        }

        let castling_availibility = fen_parts[2];
        for availibility in castling_availibility.chars() {
            if availibility == '-' {
                break;
            }

            let piece = ColoredPiece::from_fen(availibility)?;
            match (piece.color, piece.piece) {
                (Color::Black, Piece::Queen) => board.black_queenside = true,
                (Color::White, Piece::Queen) => board.white_queenside = true,
                (Color::Black, Piece::King) => board.black_kingside = true,
                (Color::White, Piece::King) => board.white_kingside = true,
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

            let to_capture_rank = match board.active {
                Color::White => to_move_rank - 1,
                Color::Black => to_move_rank + 1,
            };
            let to_capture = Square::new(to_capture_rank, file);

            board.en_passant = Some(EnPassant::new(to_move, to_capture));
        }

        let halfemoves = fen_parts[4].parse::<u16>()?;
        board.halfmoves = halfemoves;

        let fullmoves = fen_parts[5].parse::<u16>()?;
        board.fullmoves = fullmoves;

        let hash = board.board_hash();
        board.hash = hash;

        Ok(board)
    }
}
