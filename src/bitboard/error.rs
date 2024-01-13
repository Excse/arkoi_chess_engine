use thiserror::Error;

#[derive(Debug, Error)]
#[error(transparent)]
pub enum SquareError {
    InvalidSquare(#[from] InvalidSquareFormat),
}

#[derive(Debug, Error)]
#[error("the given square '{square}' is not in a valid format")]
pub struct InvalidSquareFormat {
    square: String,
}

impl InvalidSquareFormat {
    pub fn new(square: impl Into<String>) -> Self {
        Self {
            square: square.into(),
        }
    }
}
