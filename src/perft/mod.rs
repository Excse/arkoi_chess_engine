use std::ops::AddAssign;

use crate::{
    board::{zobrist::ZobristHasher, Board},
    hashtable::{
        perft::{PerftEntry, PerftStatsEntry},
        HashTable,
    },
};

mod tests;

#[derive(Default, Debug, Copy, Clone)]
pub struct PerftStats {
    nodes: u64,
    captures: u64,
    en_passants: u64,
    castles: u64,
    promotions: u64,
}

impl AddAssign for PerftStats {
    fn add_assign(&mut self, other: Self) {
        self.nodes += other.nodes;
        self.captures += other.captures;
        self.en_passants += other.en_passants;
        self.castles += other.castles;
        self.promotions += other.promotions;
    }
}

pub fn divide<const HASHED: bool>(
    board: &Board,
    hasher: &ZobristHasher,
    cache: &mut HashTable<PerftEntry>,
    depth: u8,
) -> u64 {
    let move_state = board.get_legal_moves().unwrap();

    let mut total_nodes = 0;
    for mov in move_state.moves {
        let mut board = board.clone();
        board.make(&mov).unwrap();

        let nodes = perft_normal::<HASHED>(&board, hasher, cache, depth - 1);
        total_nodes += nodes;

        println!("{} {}", mov, nodes);
    }

    println!();
    println!("{}", total_nodes);

    total_nodes
}

pub fn perft_normal<const HASHED: bool>(
    board: &Board,
    hasher: &ZobristHasher,
    cache: &mut HashTable<PerftEntry>,
    depth: u8,
) -> u64 {
    if depth == 0 {
        return 1;
    }

    let hash = board.hash ^ hasher.depth[depth as usize];
    if HASHED {
        if let Some(hashed) = cache.probe(hash) {
            return hashed.nodes;
        }
    }

    let move_state = board.get_legal_moves().unwrap();
    if move_state.is_stalemate || move_state.is_checkmate {
        return 0;
    }

    if depth == 1 {
        let moves = move_state.moves.len() as u64;
        if HASHED {
            cache.store(PerftEntry::new(hash, depth, moves));
        }

        return moves;
    }

    let mut nodes = 0;
    for mov in move_state.moves {
        let mut board = board.clone();
        board.make(&mov).unwrap();

        let next_nodes = perft_normal::<HASHED>(&board, hasher, cache, depth - 1);
        nodes += next_nodes;
    }

    if HASHED {
        cache.store(PerftEntry::new(hash, depth, nodes));
    }

    nodes
}

#[allow(dead_code)]
pub fn perft_stats<const HASHED: bool>(
    board: &Board,
    hasher: &ZobristHasher,
    cache: &mut HashTable<PerftStatsEntry>,
    depth: u8,
) -> PerftStats {
    if depth == 0 {
        let mut stats = PerftStats::default();
        stats.nodes += 1;
        return stats;
    }

    let hash = board.hash ^ hasher.depth[depth as usize];
    if HASHED {
        if let Some(hashed) = cache.probe(hash) {
            return hashed.stats;
        }
    }

    let move_state = board.get_legal_moves().unwrap();
    if move_state.is_stalemate || move_state.is_checkmate {
        return PerftStats::default();
    }

    let mut stats = PerftStats::default();
    if depth == 1 {
        let moves = move_state.moves.len() as u64;
        stats.nodes = moves;

        for mov in move_state.moves {
            if mov.is_castling() {
                stats.castles += 1;
            } else if mov.is_en_passant() {
                stats.en_passants += 1;
            } else if mov.is_promotion() {
                stats.promotions += 1;
            }

            if mov.is_capture() {
                stats.captures += 1;
            }
        }

        if HASHED {
            cache.store(PerftStatsEntry::new(hash, depth, stats));
        }

        return stats;
    }

    for mov in move_state.moves {
        let mut board = board.clone();
        board.make(&mov).unwrap();

        let next_nodes = perft_stats::<HASHED>(&board, hasher, cache, depth - 1);
        stats += next_nodes;
    }

    if HASHED {
        cache.store(PerftStatsEntry::new(hash, depth, stats));
    }

    stats
}
