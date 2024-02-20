use crossbeam_channel::{RecvError, SendError};
use thiserror::Error;

use crate::{board::error::BoardError, search::error::SearchError};

use super::parser::UCICommand;

#[derive(Debug, Error)]
#[error(transparent)]
pub enum UCIError {
    UnknownCommand(#[from] UnknownCommand),
    IOError(#[from] std::io::Error),
    NotEnoughArguments(#[from] NotEnoughArguments),
    InvalidArgument(#[from] InvalidArgument),
    ParseIntError(#[from] std::num::ParseIntError),
    BoardError(#[from] BoardError),
    SendError(#[from] SendError<UCICommand>),
    RecvError(#[from] RecvError),
    SearchError(#[from] SearchError),
    FmtError(#[from] std::fmt::Error),
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
