use thiserror::Error;

use crate::r#move::error::MoveError;

#[derive(Debug, Error)]
#[error(transparent)]
pub enum PolyglotError {
    InvalidPromotion(#[from] InvalidPromotion),
    NoEntries(#[from] NoEntries),
    InvalidData(#[from] InvalidData),
    MoveError(#[from] MoveError),
    IOError(#[from] std::io::Error),
}

#[derive(Debug, Error)]
#[error("the given promotion '{index}' is not valid")]
pub struct InvalidPromotion {
    index: u8,
}

impl InvalidPromotion {
    pub fn new(index: u8) -> Self {
        Self { index }
    }
}

#[derive(Debug, Error)]
#[error("no entries found")]
pub struct NoEntries;

#[derive(Debug, Error)]
#[error("the provided data is not divisible by 16")]
pub struct InvalidData;
