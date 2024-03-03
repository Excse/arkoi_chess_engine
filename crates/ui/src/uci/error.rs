use crossbeam_channel::{RecvError, SendError};
use thiserror::Error;

use base::{board::error::BoardError, polyglot::error::PolyglotError, r#move::error::MoveError};
use engine::search::error::SearchError;

use super::parser::UCICommand;

#[derive(Debug, Error)]
#[error(transparent)]
pub enum UCIError {
    UnknownCommand(#[from] UnknownCommand),
    IOError(#[from] std::io::Error),
    NotEnoughArguments(#[from] NotEnoughArguments),
    OptionValueMissing(#[from] OptionValueMissing),
    InvalidArgument(#[from] InvalidArgument),
    ParseIntError(#[from] std::num::ParseIntError),
    ParseBoolError(#[from] std::str::ParseBoolError),
    BoardError(#[from] BoardError),
    PolyglotError(#[from] PolyglotError),
    SendCommandError(#[from] SendError<UCICommand>),
    RecvError(#[from] RecvError),
    SearchError(#[from] SearchError),
    FmtError(#[from] std::fmt::Error),
    MoveError(#[from] MoveError),
}

#[derive(Debug, Error)]
#[error("the command '{cmd}' is unknown")]
pub struct UnknownCommand {
    cmd: String,
}

impl UnknownCommand {
    pub fn new(cmd: impl Into<String>) -> Self {
        Self { cmd: cmd.into() }
    }
}

#[derive(Debug, Error)]
#[error("you did not provide enough argument with the command '{cmd}'")]
pub struct NotEnoughArguments {
    cmd: String,
}

impl NotEnoughArguments {
    pub fn new(cmd: impl Into<String>) -> Self {
        Self { cmd: cmd.into() }
    }
}

#[derive(Debug, Error)]
#[error("passed an invalid argument '{argument}'")]
pub struct InvalidArgument {
    argument: String,
}

impl InvalidArgument {
    pub fn new(argument: impl Into<String>) -> Self {
        Self {
            argument: argument.into(),
        }
    }
}

#[derive(Debug, Error)]
#[error("the option value is missing")]
pub struct OptionValueMissing;
