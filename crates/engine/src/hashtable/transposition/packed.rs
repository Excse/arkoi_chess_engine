use base::{
    r#move::{constants::NULL_MOVE, Move},
    zobrist::ZobristHash,
};

use super::entry::{TranspositionEntry, TranspositionFlag};

pub(crate) const DEPTH_MASK: u64 = 0xFF;
pub(crate) const DEPTH_SHIFT: u64 = 0;

pub(crate) const FLAG_MASK: u64 = 0xFF;
pub(crate) const FLAG_SHIFT: u64 = 8;

pub(crate) const EVAL_MASK: u64 = 0xFFFFFFFF;
pub(crate) const EVAL_SHIFT: u64 = 16;

pub(crate) const BEST_MOVE_MASK: u64 = 0xFFFF;
pub(crate) const BEST_MOVE_SHIFT: u64 = 48;

pub(crate) const NULL_ENTRY: PackedEntry = PackedEntry::null_entry();

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PackedEntry {
    pub(super) key: ZobristHash,
    pub(super) data: u64,
}

impl PackedEntry {
    pub const fn null_entry() -> Self {
        Self {
            key: ZobristHash::new(0),
            data: 0,
        }
    }

    pub fn pack(key: ZobristHash, entry: TranspositionEntry) -> Self {
        let mut data = 0u64;

        let best_move = match entry.best_move() {
            Some(best_move) => best_move.bits(),
            None => NULL_MOVE.bits(),
        };

        data |= ((entry.depth() as u64) & DEPTH_MASK) << DEPTH_SHIFT;
        data |= ((entry.flag() as u64) & FLAG_MASK) << FLAG_SHIFT;
        data |= ((entry.eval() as u64) & EVAL_MASK) << EVAL_SHIFT;
        data |= ((best_move as u64) & BEST_MOVE_MASK) << BEST_MOVE_SHIFT;

        let actual_key = key ^ data;
        Self {
            key: actual_key,
            data,
        }
    }

    pub fn unpack(&self) -> TranspositionEntry {
        let depth = ((self.data >> DEPTH_SHIFT) & DEPTH_MASK) as u8;

        let flag = ((self.data >> FLAG_SHIFT) & FLAG_MASK) as u8;
        let flag = TranspositionFlag::from_flag(flag);

        let eval = ((self.data >> EVAL_SHIFT) & EVAL_MASK) as i32;

        let best_move = ((self.data >> BEST_MOVE_SHIFT) & BEST_MOVE_MASK) as u16;
        let best_move = if best_move != NULL_MOVE.bits() {
            Some(Move::from_bits(best_move))
        } else {
            None
        };

        TranspositionEntry::new(depth, flag, eval, best_move)
    }
}

impl PartialEq<PackedEntry> for &mut PackedEntry {
    fn eq(&self, other: &PackedEntry) -> bool {
        self.key == other.key && self.data == other.data
    }
}
