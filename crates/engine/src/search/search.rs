use std::{fmt::Write, time::Instant};

use base::{board::Board, r#move::Move};

use crate::hashtable::{HashTableStats, TranspositionTable};

use super::{error::SearchError, iterative::iterative_deepening, killers::Killers};

pub const MAX_DEPTH: u8 = 64;

pub(crate) const CHECKMATE: i32 = 1_000_000;
pub(crate) const CHECKMATE_MIN: i32 = CHECKMATE - MAX_DEPTH as i32;
pub(crate) const DRAW: i32 = 0;

pub(crate) const MAX_EVAL: i32 = CHECKMATE + 1;
pub(crate) const MIN_EVAL: i32 = -CHECKMATE - 1;

pub(crate) const NULL_DEPTH_REDUCTION: u8 = 3;

#[derive(Debug)]
pub enum StopReason {
    TimeUp,
    NodesExceeded,
}

#[derive(Debug)]
pub struct SearchStats {
    pub(crate) nodes: usize,
    pub(crate) table: HashTableStats,
    depth: u8,
    ply: u8,
    max_ply: u8,
}

impl SearchStats {
    pub fn new(depth: u8) -> Self {
        Self {
            nodes: 0,
            table: HashTableStats::default(),
            depth,
            ply: 0,
            max_ply: 0,
        }
    }

    pub fn make_search(&mut self, reduction: u8) {
        self.ply += 1;
        self.depth -= reduction;

        self.max_ply = self.max_ply.max(self.ply);
    }

    pub fn unmake_search(&mut self, reduction: u8) {
        self.ply -= 1;
        self.depth += reduction;
    }

    #[inline(always)]
    pub const fn is_leaf(&self) -> bool {
        self.depth == 0
    }

    #[inline(always)]
    pub fn extend_search(&mut self) {
        self.depth += 1;
    }

    #[inline(always)]
    pub const fn depth(&self) -> u8 {
        self.depth
    }

    #[inline(always)]
    pub const fn ply(&self) -> u8 {
        self.ply
    }
}

#[derive(Debug)]
pub struct SearchInfo {
    pub(crate) board: Board,
    pub(crate) time_frame: TimeFrame,
    pub(crate) accumulated_nodes: usize,
    pub(crate) max_nodes: usize,
    pub(crate) max_depth: u8,
    // TODO: Use the given moves
    pub(crate) _moves: Vec<Move>,
    pub(crate) infinite: bool,
    pub(crate) killers: Killers,
    pub(crate) mate_killers: Killers,
}

impl SearchInfo {
    pub fn new(
        board: Board,
        move_time: u128,
        max_nodes: usize,
        max_depth: u8,
        moves: Vec<Move>,
        infinite: bool,
    ) -> Self {
        assert!(max_depth <= MAX_DEPTH);

        let time_frame = TimeFrame::new(move_time);
        SearchInfo {
            board,
            time_frame,
            accumulated_nodes: 0,
            max_nodes,
            max_depth,
            _moves: moves,
            infinite,
            killers: Killers::default(),
            mate_killers: Killers::default(),
        }
    }
}

#[derive(Debug)]
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
}

pub fn search<W: Write>(
    cache: &TranspositionTable,
    search_info: SearchInfo,
    output: &mut W,
) -> Result<Move, SearchError> {
    let best_move = iterative_deepening(cache, search_info, output)?;
    Ok(best_move)
}

pub fn should_stop_search(info: &SearchInfo, stats: &SearchStats) -> Result<(), StopReason> {
    let elapsed = info.time_frame.start_time.elapsed().as_millis();
    if elapsed >= info.time_frame.move_time {
        return Err(StopReason::TimeUp);
    }

    let nodes = info.accumulated_nodes + stats.nodes;
    if nodes >= info.max_nodes {
        return Err(StopReason::NodesExceeded);
    }

    Ok(())
}
