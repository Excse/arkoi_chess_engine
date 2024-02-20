use std::{
    io::{stdin, stdout},
    path::Path,
    thread,
    time::Instant,
};

use clap::{Parser, Subcommand};
use parse_size::parse_size;

use board::{zobrist::ZobristHasher, Board};
use hashtable::HashTable;
use uci::{controller::UCIController, parser::UCIParser};

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
    let (sender, receiver) = crossbeam_channel::unbounded();

    let controller = thread::spawn(move || {
        let writer = stdout().lock();

        let mut uci_controller = UCIController::new(writer, receiver, cache_size);
        // TODO: Check what to do with the result
        uci_controller.start().unwrap();
    });

    let parser = thread::spawn(move || {
        let reader = stdin().lock();

        let mut uci_input = UCIParser::new(reader, sender);
        // TODO: Check what to do with the result
        uci_input.start().unwrap();
    });

    parser.join().expect("Couldnt join the parser thread");
    controller
        .join()
        .expect("Couldnt join the controller thread");

    Ok(())
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

    let mut board = Board::from_str(&fen, hasher)?;
    board.make_moves(&moves)?;

    let mut cache = HashTable::size(cache_size);

    let start = Instant::now();

    let nodes = if divide {
        if hashed {
            perft::divide::<true>(&mut board, &mut cache, depth)
        } else {
            perft::divide::<false>(&mut board, &mut cache, depth)
        }
    } else {
        if hashed {
            perft::perft_normal::<true>(&mut board, &mut cache, depth)
        } else {
            perft::perft_normal::<false>(&mut board, &mut cache, depth)
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
