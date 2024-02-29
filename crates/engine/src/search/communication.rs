use std::num::NonZeroU8;

use base::r#move::Move;

#[derive(Debug, Clone)]
pub enum SearchCommand {
    BestMove(BestMove),
    Info(Info),
}

#[derive(Debug, Copy, Clone)]
pub struct BestMove {
    pub mov: Move,
}

impl BestMove {
    pub fn new(mov: Move) -> SearchCommand {
        SearchCommand::BestMove(Self { mov })
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Score {
    Centipawns(i32),
    Mate(i32),
}

#[derive(Debug, Clone)]
pub struct Info {
    pub depth: Option<u8>,
    pub seldepth: Option<u8>,
    pub time: Option<u128>,
    pub nodes: Option<usize>,
    pub pv: Option<Vec<Move>>,
    pub score: Option<Score>,
    pub currmove: Option<Move>,
    pub currmovenumber: Option<NonZeroU8>,
    pub hashfull: Option<u16>,
    pub nps: Option<u64>,
    pub string: Option<String>,
}

impl Info {
    pub fn new() -> Self {
        Self {
            depth: None,
            seldepth: None,
            time: None,
            nodes: None,
            pv: None,
            score: None,
            currmove: None,
            currmovenumber: None,
            hashfull: None,
            nps: None,
            string: None,
        }
    }

    pub fn depth(mut self, depth: u8) -> Self {
        self.depth = Some(depth);
        self
    }

    pub fn seldepth(mut self, seldepth: u8) -> Self {
        self.seldepth = Some(seldepth);
        self
    }

    pub fn time(mut self, time: u128) -> Self {
        self.time = Some(time);
        self
    }

    pub fn nodes(mut self, nodes: usize) -> Self {
        self.nodes = Some(nodes);
        self
    }

    pub fn pv(mut self, pv: Vec<Move>) -> Self {
        self.pv = Some(pv);
        self
    }

    pub fn score(mut self, score: Score) -> Self {
        self.score = Some(score);
        self
    }

    pub fn currmove(mut self, currmove: Move) -> Self {
        self.currmove = Some(currmove);
        self
    }

    pub fn currmovenumber(mut self, currmovenumber: NonZeroU8) -> Self {
        self.currmovenumber = Some(currmovenumber);
        self
    }

    pub fn hashfull(mut self, hashfull: u16) -> Self {
        self.hashfull = Some(hashfull);
        self
    }

    pub fn nps(mut self, nps: u64) -> Self {
        self.nps = Some(nps);
        self
    }

    pub fn string(mut self, string: String) -> Self {
        self.string = Some(string);
        self
    }

    pub fn build(self) -> SearchCommand {
        SearchCommand::Info(self)
    }
}
