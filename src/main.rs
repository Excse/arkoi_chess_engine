use std::{
    io::{stdin, stdout},
    path::Path,
    time::Instant,
};

use clap::{Parser, Subcommand};
use parse_size::parse_size;

use crate::{evaluation::evaluate, search::search};
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
        #[clap(long, short, default_value = "4GiB")]
        cache_size: u64,
    },
    Perft {
        #[clap(long, short)]
        more_information: bool,
        #[clap(long, short)]
        divide: bool,
        #[clap(value_parser = parse_cache_size)]
        #[clap(long, short, default_value = "1GiB")]
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
            more_information,
            divide,
        } => perft_command(
            depth,
            fen,
            cache_size as usize,
            moves,
            more_information,
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
                println!("Hash: 0x{:X}", board.gamestate.hash.0);

                let move_state = board.get_legal_moves()?;
                println!("Moves {}:", move_state.moves.len());
                print!(" - ");
                for mov in move_state.moves {
                    print!("{}, ", mov);
                }
                println!();
                println!("Checkmate: {}", move_state.is_checkmate);
                println!("Stalemate: {}", move_state.is_stalemate);
                println!(
                    "Evaluation for side to move: {}",
                    evaluate(&board, board.gamestate.active)
                );

                if let Some(en_passant) = &board.gamestate.en_passant {
                    println!(
                        "En passant: Capture {} and move to {}",
                        en_passant.to_capture, en_passant.to_move
                    );
                }
            }
            Command::Go(command) => {
                let best_move = search(&mut board, &mut cache, &command)?;
                uci.send_bestmove(&mut writer, &best_move)?;
                board.make(&best_move);
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
    divide: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut rand = rand::thread_rng();
    let hasher = ZobristHasher::new(&mut rand);

    let mut board = Board::from_str(&fen, &hasher)?;
    board.make_moves(&moves)?;

    let mut cache = HashTable::size(cache_size);

    let start = Instant::now();

    let nodes = if divide {
        perft::divide::<true>(&mut board, &hasher, &mut cache, depth)
    } else {
        perft::perft_normal::<true>(&mut board, &hasher, &mut cache, depth)
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
