use base::board::error::BoardError;
use thiserror::Error;

use crate::generator::error::MoveGeneratorError;

#[derive(Debug, Error)]
#[error(transparent)]
pub enum SearchError {
    MoveGeneratorError(#[from] MoveGeneratorError),
    BoardError(#[from] BoardError),
}
