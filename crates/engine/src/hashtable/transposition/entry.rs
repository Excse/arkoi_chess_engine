use base::r#move::{constants::NULL_MOVE, Move};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TranspositionFlag {
    Exact,
    LowerBound,
    UpperBound,
}

impl TranspositionFlag {
    pub fn from_flag(flag: u8) -> Self {
        match flag {
            0 => TranspositionFlag::Exact,
            1 => TranspositionFlag::LowerBound,
            2 => TranspositionFlag::UpperBound,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranspositionEntry {
    depth: u8,
    flag: TranspositionFlag,
    eval: i32,
    best_move: Move,
}

// We need to ensure that the size of the struct is 8 byte or less.
pub const _: () = assert!(std::mem::size_of::<TranspositionEntry>() <= 8);

impl TranspositionEntry {
    pub fn new(depth: u8, flag: TranspositionFlag, eval: i32, best_move: Option<Move>) -> Self {
        let best_move = if best_move.is_some() {
            best_move.unwrap()
        } else {
            NULL_MOVE
        };

        Self {
            depth,
            flag,
            eval,
            best_move,
        }
    }

    #[inline(always)]
    pub const fn depth(&self) -> u8 {
        self.depth
    }

    #[inline(always)]
    pub const fn eval(&self) -> i32 {
        self.eval
    }

    #[inline(always)]
    pub fn best_move(&self) -> Option<Move> {
        if self.best_move == NULL_MOVE {
            return None;
        }

        Some(self.best_move)
    }

    #[inline(always)]
    pub fn flag(&self) -> TranspositionFlag {
        self.flag
    }
}
