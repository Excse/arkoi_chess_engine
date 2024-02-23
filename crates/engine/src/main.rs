use std::env;

use base::{board::Board, zobrist::ZobristHasher};
use engine::{hashtable::GenericTable, perft::divide};
use rand::{rngs::StdRng, SeedableRng};

pub mod evaluation;
pub mod generator;
pub mod hashtable;
pub mod perft;
pub mod search;

fn main() {
    let args: Vec<String> = env::args().collect();

    // Check if there are enough arguments
    if args.len() < 3 {
        eprintln!("Usage: {} <arg1> <arg2> <arg3>", args[0]);
        println!("arg1: Depth");
        println!("arg2: FEN string");
        println!("arg3: Optional moves");
        println!("Given arguments: {:?}", args);
        std::process::exit(1);
    }

    // Parse arguments
    let depth = match args[1].parse::<u8>() {
        Ok(n) => n,
        Err(_) => {
            eprintln!("Error: Could not parse depth {}", args[1]);
            std::process::exit(1);
        }
    };
    let fen = &args[2];

    let mut rand = StdRng::seed_from_u64(42);
    let hasher = ZobristHasher::new(&mut rand);

    let mut board = Board::from_str(fen, hasher).unwrap();

    if args.len() == 4 {
        let moves = args[3]
            .split(" ")
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        board.make_moves(&moves).unwrap();
    }

    let mut cache = GenericTable::size(67108864);
    divide::<true>(&mut board, &mut cache, depth);
}
