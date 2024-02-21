use api::board::error::BoardError;
use thiserror::Error;

#[derive(Debug, Error)]
#[error(transparent)]
pub enum MoveGeneratorError {
    BoardError(#[from] BoardError),
}
