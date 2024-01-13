use thiserror::Error;

pub type Result<T> = std::result::Result<T, UCIError>;

#[derive(Debug, Error)]
#[error(transparent)]
pub enum UCIError {
    UnknownCommand(#[from] UnknownCommand),
    IOError(#[from] std::io::Error),
    NotEnoughArguments(#[from] NotEnoughArguments),
    InvalidMove(#[from] InvalidMove),
    InvalidArgument(#[from] InvalidArgument),
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
#[error("the given move '{mov}' is not in a valid long algebraric notation")]
pub struct InvalidMove {
    mov: String,
}

impl InvalidMove {
    pub fn new(mov: impl Into<String>) -> Self {
        Self { mov: mov.into() }
    }
}

#[derive(Debug, Error)]
#[error("passed an invalid argument '{argument}'")]
pub struct InvalidArgument {
    argument: String,
}

impl InvalidArgument {
    pub fn new(argument: &'static str) -> Self {
        Self {
            argument: argument.into(),
        }
    }
}
