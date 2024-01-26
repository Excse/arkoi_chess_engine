use thiserror::Error;

use crate::{bitboard::error::SquareError, board::error::ColoredPieceError};

#[derive(Debug, Error)]
#[error(transparent)]
pub enum MoveError {
    InvalidMoveFormat(#[from] InvalidMoveFormat),
    PieceNotFound(#[from] PieceNotFound),
    SquareError(#[from] SquareError),
    ColoredPieceError(#[from] ColoredPieceError),
}

#[derive(Debug, Error)]
#[error("the given move '{mov}' is not in a valid format")]
pub struct InvalidMoveFormat {
    mov: String,
}

impl InvalidMoveFormat {
    pub fn new(mov: impl Into<String>) -> Self {
        Self { mov: mov.into() }
    }
}

#[derive(Debug, Error)]
#[error("couldnt find a piece on the square '{square}'")]
pub struct PieceNotFound {
    square: String,
}

impl PieceNotFound {
    pub fn new(square: impl Into<String>) -> Self {
        Self {
            square: square.into(),
        }
    }
}
