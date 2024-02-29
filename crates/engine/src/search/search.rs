use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
    u128,
};

use base::{board::Board, r#move::Move};
use crossbeam_channel::Sender;

use crate::hashtable::TranspositionTable;

use super::{
    communication::SearchCommand, error::SearchError, iterative::iterative_deepening,
    killers::Killers,
};

pub const MAX_DEPTH: u8 = 64;

pub(crate) const CHECK_TERMINATION: usize = 0x7FF;
pub(crate) const SEND_STATS: usize = 0x7FFFF;

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
    ForcedStop,
}

#[derive(Debug)]
pub struct SearchStats {
    pub(crate) nodes: usize,
    pub(crate) quiescence_nodes: usize,
    pub(crate) start_time: Instant,
    depth: u8,
    ply: u8,
    max_ply: u8,
}

impl SearchStats {
    pub fn new(depth: u8) -> Self {
        assert!(depth <= MAX_DEPTH);

        Self {
            nodes: 0,
            quiescence_nodes: 0,
            depth,
            ply: 0,
            max_ply: 0,
            start_time: Instant::now(),
        }
    }

    pub fn make_search(&mut self, reduction: u8) {
        self.increase_ply();
        self.decrease_depth(reduction);
    }

    pub fn unmake_search(&mut self, reduction: u8) {
        self.decrease_ply();
        self.increase_depth(reduction);
    }

    pub fn increase_ply(&mut self) {
        self.ply += 1;
        self.max_ply = self.max_ply.max(self.ply);
    }

    pub fn decrease_ply(&mut self) {
        self.ply -= 1;
    }

    pub fn increase_depth(&mut self, reduction: u8) {
        debug_assert!(self.depth + reduction <= MAX_DEPTH);

        self.depth += reduction;
    }

    pub fn decrease_depth(&mut self, reduction: u8) {
        debug_assert!(self.depth >= reduction);

        self.depth -= reduction;
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
    pub(crate) sender: Sender<SearchCommand>,
    pub(crate) running: Arc<AtomicBool>,
    pub(crate) time_frame: TimeFrame,
    pub(crate) accumulated_nodes: usize,
    pub(crate) max_nodes: Option<usize>,
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
        sender: Sender<SearchCommand>,
        running: Arc<AtomicBool>,
        time_frame: TimeFrame,
        max_nodes: Option<usize>,
        max_depth: Option<u8>,
        moves: Vec<Move>,
        infinite: bool,
    ) -> Self {
        SearchInfo {
            board,
            sender,
            running,
            time_frame,
            accumulated_nodes: 0,
            max_nodes,
            max_depth: max_depth.unwrap_or(MAX_DEPTH),
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
    pub(crate) move_time: u128,
}

impl TimeFrame {
    pub fn new(move_time: u128) -> Self {
        TimeFrame {
            start_time: Instant::now(),
            move_time,
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn estimate(time_left: u128, increment: u128) -> Self {
        let mut time = (time_left / 40) + (increment / 2);

        if time >= time_left {
            time = (time_left / (increment + 1)) * time_left;
        }

        Self::new(time)
    }
}

// TODO: Add LazySMP
pub fn search(cache: &TranspositionTable, search_info: SearchInfo) -> Result<(), SearchError> {
    iterative_deepening(cache, search_info)?;
    Ok(())
}

pub fn should_stop_search(info: &SearchInfo, stats: &SearchStats) -> Result<(), StopReason> {
    if let Some(max_nodes) = info.max_nodes {
        let nodes = info.accumulated_nodes + stats.nodes;
        if nodes >= max_nodes {
            return Err(StopReason::NodesExceeded);
        }
    }

    if !info.running.load(Ordering::Relaxed) {
        return Err(StopReason::ForcedStop);
    }

    let elapsed = info.time_frame.start_time.elapsed().as_millis();
    if elapsed >= info.time_frame.move_time {
        return Err(StopReason::TimeUp);
    }

    Ok(())
}
