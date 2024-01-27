use std::num::ParseIntError;

use thiserror::Error;

#[derive(Debug, Error)]
#[error(transparent)]
pub enum ColoredPieceError {
    InvalidFenPiece(#[from] InvalidFenPiece),
}

#[derive(Debug, Error)]
#[error("the given piece '{piece}' is not valid. You can only use 'k', 'q', 'r', 'p', 'b', 'n' in upper or lower case")]
pub struct InvalidFenPiece {
    piece: char,
}

impl InvalidFenPiece {
    pub fn new(piece: char) -> Self {
        Self { piece }
    }
}

#[derive(Debug, Error)]
#[error(transparent)]
pub enum BoardError {
    NotEnoughParts(#[from] NotEnoughParts),
    WrongActiveColor(#[from] WrongActiveColor),
    WrongCastlingAvailibility(#[from] WrongCastlingAvailibility),
    InvalidEnPassant(#[from] InvalidEnPassant),
    PieceNotFound(#[from] PieceNotFound),
    ParseInt(#[from] ParseIntError),
    ColoredPieceError(#[from] ColoredPieceError),
    MultipleKings(#[from] MultipleKings),
}

#[derive(Debug, Error)]
#[error("there are not enough parts for this FEN")]
pub struct NotEnoughParts;

#[derive(Debug, Error)]
#[error("the active color '{given}' is not valid. You can only use 'w' or 'b'")]
pub struct WrongActiveColor {
    given: String,
}

impl WrongActiveColor {
    pub fn new(given: impl Into<String>) -> Self {
        Self {
            given: given.into(),
        }
    }
}

#[derive(Debug, Error)]
#[error("the castling availibilty '{given}' is not valid. You can only use 'Q', 'K', 'q' or 'k'")]
pub struct WrongCastlingAvailibility {
    given: char,
}

impl WrongCastlingAvailibility {
    pub fn new(given: char) -> Self {
        Self { given }
    }
}

#[derive(Debug, Error)]
#[error("the given en passant square '{square}' is not valid")]
pub struct InvalidEnPassant {
    square: String,
}

impl InvalidEnPassant {
    pub fn new(square: impl Into<String>) -> Self {
        Self {
            square: square.into(),
        }
    }
}

#[derive(Debug, Error)]
#[error("couldn't find the piece for this move")]
pub struct PieceNotFound;

#[derive(Debug, Error)]
#[error("there are multiple kings for the same color")]
pub struct MultipleKings {
    amount: usize,
}

impl MultipleKings {
    pub fn new(amount: usize) -> Self {
        Self { amount }
    }
}
