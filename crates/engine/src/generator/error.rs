use thiserror::Error;

use base::board::error::BoardError;

#[derive(Debug, Error)]
#[error(transparent)]
pub enum MoveGeneratorError {
    BoardError(#[from] BoardError),
}
