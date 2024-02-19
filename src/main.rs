use std::{
    io::{stdin, stdout},
    path::Path,
    time::Instant,
};

use clap::{Parser, Subcommand};
use parse_size::parse_size;

use crate::{evaluation::evaluate, generation::MoveGenerator, search::search};
use board::{zobrist::ZobristHasher, Board};
use hashtable::HashTable;
use uci::{commands::Command, UCI};

mod bitboard;
mod board;
mod evaluation;
mod generation;
mod hashtable;
mod lookup;
mod perft;
mod search;
mod uci;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct CLI {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand)]
enum CliCommand {
    UCI {
        #[clap(value_parser = parse_cache_size)]
        #[clap(long, default_value = "1GiB")]
        cache_size: u64,
    },
    Perft {
        #[clap(long)]
        more_information: bool,
        #[clap(long)]
        divide: bool,
        #[clap(long)]
        hashed: bool,
        #[clap(value_parser = parse_cache_size)]
        #[clap(long, default_value = "1GiB")]
        cache_size: u64,
        depth: u8,
        fen: String,
        #[clap(value_parser, num_args = 0.., value_delimiter = ' ')]
        moves: Vec<String>,
    },
    TableGenerator,
}

fn parse_cache_size(arg: &str) -> Result<u64, String> {
    parse_size(arg).map_err(|err| err.to_string())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = CLI::parse();
    match cli.command {
        CliCommand::UCI { cache_size } => uci_command(cache_size as usize),
        CliCommand::Perft {
            depth,
            fen,
            cache_size,
            moves,
            hashed,
            more_information,
            divide,
        } => perft_command(
            depth,
            fen,
            cache_size as usize,
            moves,
            more_information,
            hashed,
            divide,
        ),
        CliCommand::TableGenerator => table_generator_command(),
    }
}

fn uci_command(cache_size: usize) -> Result<(), Box<dyn std::error::Error>> {
    let name = format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    let author = env!("CARGO_PKG_AUTHORS");
    let mut uci = UCI::new(name, author);

    let mut reader = stdin().lock();
    let mut writer = stdout();

    let mut rand = rand::thread_rng();
    let hasher = ZobristHasher::new(&mut rand);

    let mut cache = HashTable::size(cache_size);

    let mut board = Board::default(&hasher);
    loop {
        let result = match uci.receive_command(&mut reader, &mut writer)? {
            Some(command) => command,
            None => continue,
        };

        match result {
            Command::Position(command) => {
                board = Board::from_str(&command.fen, &hasher)?;
                board.make_moves(&command.moves)?;
            }
            Command::IsReady => {
                uci.send_readyok(&mut writer)?;
            }
            Command::Quit => {
                return Ok(());
            }
            Command::Show => {
                println!("{}", board);
                println!("FEN: {}", board.to_fen());
                println!("Hash: 0x{:X}", board.hash());

                let moves = MoveGenerator::new(&board);
                println!("Moves {}:", moves.len());
                print!(" - ");
                for mov in moves {
                    print!("{}, ", mov);
                }
                println!();
                // println!("Checkmate: {}", moves.is_checkmate());
                // println!("Stalemate: {}", moves.is_stalemate());
                println!(
                    "Evaluation for side to move: {}",
                    evaluate(&board, board.active())
                );

                if let Some(en_passant) = board.en_passant() {
                    println!(
                        "En passant: Capture {} and move to {}",
                        en_passant.to_capture, en_passant.to_move
                    );
                }
            }
            Command::Go(command) => {
                let best_move = search(&mut board, &mut cache, &command)?;
                uci.send_bestmove(&mut writer, &best_move)?;
            }
            Command::CacheStats => {
                let probes = cache.hits() + cache.misses();
                let hit_rate = cache.hits() as f64 / probes as f64;
                println!("Statistics of the cache usage:");
                println!(" - Hit rate: {:.2}%:", hit_rate * 100.0);
                println!("    - Overall probes: {}", probes);
                println!("    - Hits: {}", cache.hits());
                println!("    - Misses: {}", cache.misses());

                let stores = cache.new() + cache.overwrites();
                let overwrite_rate = cache.overwrites() as f64 / stores as f64;
                println!(" - Overwrite rate: {:.2}%:", overwrite_rate * 100.0);
                println!("    - Overall stores: {}", stores);
                println!("    - New: {}", cache.new());
                println!("    - Overwrites: {}", cache.overwrites());
            }
        }
    }
}

fn perft_command(
    depth: u8,
    fen: String,
    cache_size: usize,
    moves: Vec<String>,
    more_information: bool,
    hashed: bool,
    divide: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut rand = rand::thread_rng();
    let hasher = ZobristHasher::new(&mut rand);

    let mut board = Board::from_str(&fen, &hasher)?;
    board.make_moves(&moves)?;

    let mut cache = HashTable::size(cache_size);

    let start = Instant::now();

    let nodes = if divide {
        if hashed {
            perft::divide::<true>(&mut board, &hasher, &mut cache, depth)
        } else {
            perft::divide::<false>(&mut board, &hasher, &mut cache, depth)
        }
    } else {
        if hashed {
            perft::perft_normal::<true>(&mut board, &hasher, &mut cache, depth)
        } else {
            perft::perft_normal::<false>(&mut board, &hasher, &mut cache, depth)
        }
    };

    if more_information {
        let end = Instant::now();
        let duration = end.duration_since(start);

        let nodes_per_second = nodes as f64 / duration.as_secs_f64();
        println!("Duration: {:?}", duration);
        println!("Nodes per second: {}", nodes_per_second);
    }

    Ok(())
}

fn table_generator_command() -> Result<(), Box<dyn std::error::Error>> {
    let mut output = String::new();
    lookup::generate_lookup_tables(&mut output)?;

    let path = Path::new("./src/lookup/tables.rs");
    std::fs::write(path, output)?;

    Ok(())
}
