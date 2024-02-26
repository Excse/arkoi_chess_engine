use std::{fmt::Write, time::Instant};

use base::{board::Board, r#move::Move};

use crate::hashtable::TranspositionTable;

use super::{
    error::{SearchError, TimeUp},
    iterative::iterative_deepening,
};

pub const MAX_DEPTH: u8 = 64;

pub(crate) const CHECKMATE: i32 = 1_000_000;
pub(crate) const CHECKMATE_MIN: i32 = CHECKMATE - MAX_DEPTH as i32;
pub(crate) const DRAW: i32 = 0;

pub(crate) const MAX_EVAL: i32 = CHECKMATE + 1;
pub(crate) const MIN_EVAL: i32 = -CHECKMATE - 1;

pub(crate) const NULL_DEPTH_REDUCTION: u8 = 2;

pub struct SearchInfo {
    time_frame: TimeFrame,
    max_nodes: usize,
    max_depth: u8,
    moves: Vec<Move>,
    infinite: bool,
}

impl SearchInfo {
    pub fn new(
        move_time: u128,
        max_nodes: usize,
        max_depth: u8,
        moves: Vec<Move>,
        infinite: bool,
    ) -> Self {
        assert!(max_depth <= MAX_DEPTH);

        let time_frame = TimeFrame::new(move_time);
        SearchInfo {
            time_frame,
            max_nodes,
            max_depth,
            moves,
            infinite,
        }
    }

    #[inline(always)]
    pub fn time_frame(&self) -> &TimeFrame {
        &self.time_frame
    }

    #[inline(always)]
    pub fn max_nodes(&self) -> usize {
        self.max_nodes
    }

    #[inline(always)]
    pub fn max_depth(&self) -> u8 {
        self.max_depth
    }

    #[inline(always)]
    pub fn moves(&self) -> &[Move] {
        &self.moves
    }

    #[inline(always)]
    pub fn is_infinite(&self) -> bool {
        self.infinite
    }
}

pub struct TimeFrame {
    start_time: Instant,
    move_time: u128,
}

impl TimeFrame {
    pub fn new(move_time: u128) -> Self {
        TimeFrame {
            start_time: Instant::now(),
            move_time,
        }
    }

    pub fn is_time_up(&self) -> Result<(), SearchError> {
        let elapsed = self.start_time.elapsed().as_millis();
        if elapsed >= self.move_time {
            return Err(TimeUp.into());
        }

        Ok(())
    }
}

pub fn search<W: Write>(
    mut board: Board,
    mut cache: TranspositionTable,
    search_info: SearchInfo,
    output: &mut W,
) -> Result<Move, SearchError> {
    let best_move = iterative_deepening(&mut board, &mut cache, search_info, output)?;
    Ok(best_move)
}
