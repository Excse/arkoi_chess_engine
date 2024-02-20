use std::ops::AddAssign;

use api::board::Board;

use crate::{
    generation::MoveGenerator,
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
    board: &mut Board,
    cache: &mut HashTable<PerftEntry>,
    depth: u8,
) -> u64 {
    let move_generator = MoveGenerator::new(board);

    let mut total_nodes = 0;
    for mov in move_generator {
        board.make(mov);

        let nodes = perft_normal::<HASHED>(board, cache, depth - 1);
        total_nodes += nodes;

        board.unmake(mov);

        println!("{} {}", mov, nodes);
    }

    println!();
    println!("{}", total_nodes);

    total_nodes
}

pub fn perft_normal<const HASHED: bool>(
    board: &mut Board,
    cache: &mut HashTable<PerftEntry>,
    depth: u8,
) -> u64 {
    if depth == 0 {
        return 1;
    }

    let hash = board.hash() ^ board.hasher().depth_hash(depth);
    if HASHED {
        if let Some(hashed) = cache.probe(hash) {
            return hashed.nodes();
        }
    }

    let move_generator = MoveGenerator::new(board);
    if depth == 1 {
        let moves = move_generator.len() as u64;
        if HASHED {
            cache.store(PerftEntry::new(hash, depth, moves));
        }

        return moves;
    }

    let mut nodes = 0;
    for mov in move_generator {
        board.make(mov);

        let next_nodes = perft_normal::<HASHED>(board, cache, depth - 1);
        nodes += next_nodes;

        board.unmake(mov);
    }

    if HASHED {
        cache.store(PerftEntry::new(hash, depth, nodes));
    }

    nodes
}

#[allow(dead_code)]
pub fn perft_stats<const HASHED: bool>(
    board: &mut Board,
    cache: &mut HashTable<PerftStatsEntry>,
    depth: u8,
) -> PerftStats {
    if depth == 0 {
        let mut stats = PerftStats::default();
        stats.nodes += 1;
        return stats;
    }

    let hash = board.hash() ^ board.hasher().depth_hash(depth);
    if HASHED {
        if let Some(hashed) = cache.probe(hash) {
            return *hashed.stats();
        }
    }

    let move_generator = MoveGenerator::new(board);

    let mut stats = PerftStats::default();
    if depth == 1 {
        let moves = move_generator.len() as u64;
        stats.nodes = moves;

        for mov in move_generator {
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

    for mov in move_generator {
        board.make(mov);

        let next_nodes = perft_stats::<HASHED>(board, cache, depth - 1);
        stats += next_nodes;

        board.unmake(mov);
    }

    if HASHED {
        cache.store(PerftStatsEntry::new(hash, depth, stats));
    }

    stats
}
