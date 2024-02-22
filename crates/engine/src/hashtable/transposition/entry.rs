use base::r#move::{constants::NULL_MOVE, Move};

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum TranspositionFlag {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Debug, Clone)]
pub struct TranspositionEntry {
    depth: u8,
    flag: TranspositionFlag,
    eval: i32,
    best_move: Move,
}

// We need to ensure that the size of the struct is 64 bit or less.
pub const _: () = assert!(std::mem::size_of::<TranspositionEntry>() <= 64);

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
