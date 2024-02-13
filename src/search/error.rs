use thiserror::Error;

use crate::{board::error::BoardError, generation::error::MoveGeneratorError};

#[derive(Debug, Error)]
#[error(transparent)]
pub enum SearchError {
    MoveGeneratorError(#[from] MoveGeneratorError),
    BoardError(#[from] BoardError),
    TimeUp(#[from] TimeUp),
    InCheckmate(#[from] InCheckmate),
}

#[derive(Debug, Error)]
#[error("time up")]
pub struct TimeUp;

#[derive(Debug, Error)]
#[error("in checkmate")]
pub struct InCheckmate;
