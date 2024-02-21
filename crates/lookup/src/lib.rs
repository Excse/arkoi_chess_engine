#![allow(long_running_const_eval)]

pub(crate) const BOARD_SIZE: usize = 64;
pub(crate) const COLOR_COUNT: usize = 2;

pub mod direction;
pub mod generic;
pub mod magic;
pub mod moves;
pub mod pesto;
pub mod utils;
