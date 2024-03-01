use base::{board::error::BoardError, polyglot::error::PolyglotError};
use crossbeam_channel::SendError;
use thiserror::Error;

use crate::generator::error::MoveGeneratorError;

use super::communication::SearchCommand;

#[derive(Debug, Error)]
#[error(transparent)]
pub enum SearchError {
    MoveGeneratorError(#[from] MoveGeneratorError),
    PolyglotError(#[from] PolyglotError),
    BoardError(#[from] BoardError),
    SendError(#[from] SendError<SearchCommand>),
}
