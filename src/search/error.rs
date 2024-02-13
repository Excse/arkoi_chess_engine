use thiserror::Error;

use crate::{board::error::BoardError, generation::error::MoveGeneratorError};

#[derive(Debug, Error)]
#[error(transparent)]
pub enum SearchError {
    NoDepthOrInfinite(#[from] NoDepthOrInfinite),
    MoveGeneratorError(#[from] MoveGeneratorError),
    BoardError(#[from] BoardError),
}

#[derive(Debug, Error)]
#[error("no depth or infinite search")]
pub struct NoDepthOrInfinite;
