use std::{
    collections::HashMap,
    io::{stdin, stdout},
    time::Instant,
};

use clap::{Parser, Subcommand};

use board::{
    color::Color,
    zobrist::{ZobristHash, ZobristHasher},
    Board,
};
use move_generator::{mov::Move, MoveGenerator};
use search::minimax;
use uci::{Command, UCI};

use crate::search::evaluate;

mod bitboard;
mod board;
mod lookup;
mod move_generator;
mod search;
mod uci;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand)]
enum CliCommand {
    UCI {
        #[clap(long, short, default_value = "6")]
        max_depth: usize,
    },
    Perft {
        #[clap(long, short)]
        more_information: bool,
        depth: usize,
        fen: String,
        #[clap(value_parser, num_args = 0.., value_delimiter = ' ')]
        moves: Vec<String>,
    },
    TableGenerator,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        CliCommand::UCI { max_depth } => uci_command(max_depth),
        CliCommand::Perft {
            depth,
            fen,
            moves,
            more_information,
        } => perft_command(depth, fen, moves, more_information),
        CliCommand::TableGenerator => table_generator_command(),
    }
}

fn uci_command(max_depth: usize) -> Result<(), Box<dyn std::error::Error>> {
    let name = format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    let author = env!("CARGO_PKG_AUTHORS");
    let mut uci = UCI::new(name, author);

    let mut reader = stdin().lock();
    let mut writer = stdout();

    let mut rand = rand::thread_rng();
    let hasher = ZobristHasher::new(&mut rand);

    let move_generator = MoveGenerator::default();
    let mut board = Board::default(&hasher);
    loop {
        let result = uci.receive_command(&mut reader, &mut writer);
        match result {
            Ok(Command::NewPosition(fen, moves)) => {
                board = Board::from_str(&fen, &hasher)?;

                for mov_str in moves {
                    let mov = Move::parse(mov_str, board.active, &board)?;
                    board.make(&mov)?;
                }
            }
            Ok(Command::IsReady) => {
                uci.send_readyok(&mut writer)?;
            }
            Ok(Command::Quit) => {
                return Ok(());
            }
            Ok(Command::Show) => {
                println!("{}", board);
                println!("FEN: {}", board.to_fen());
                println!("Hash: 0x{:X}", board.hash.0);

                let moves = move_generator.get_legal_moves(&board)?;
                println!("Moves {}:", moves.len());
                print!(" - ");
                for mov in moves {
                    print!("{}, ", mov);
                }
                println!();
                println!("Evaluation:");
                println!(" - White: {}", evaluate(&board, Color::White));
                println!(" - Black: {}", evaluate(&board, Color::Black));

                if let Some(en_passant) = board.en_passant {
                    println!(
                        "En passant: Capture {} and move to {}",
                        en_passant.to_capture, en_passant.to_move
                    );
                }
            }
            Ok(Command::Go) => {
                let (best_eval, best_move) = minimax(
                    &board,
                    &move_generator,
                    max_depth,
                    max_depth,
                    std::isize::MIN,
                    std::isize::MAX,
                    board.active,
                );
                println!("Best move {:?} with eval {}", best_move, best_eval);

                if let Some(best_move) = best_move {
                    uci.send_bestmove(&mut writer, &best_move)?;
                    board.make(&best_move)?;
                } else {
                    panic!("No best move found");
                }
            }
            Ok(Command::None) => {}
            Err(error) => eprintln!("{:?}", error),
        }
    }
}

fn perft_command(
    depth: usize,
    fen: String,
    moves: Vec<String>,
    more_information: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut rand = rand::thread_rng();
    let hasher = ZobristHasher::new(&mut rand);
    let mut board = Board::from_str(&fen, &hasher)?;

    for mov_str in moves {
        let mov = Move::parse(mov_str, board.active, &board)?;
        board.make(&mov)?;
    }

    let move_generator = MoveGenerator::default();

    let mut leaf_cache = HashMap::with_capacity(1_000_000);

    let start = Instant::now();

    let moves = move_generator.get_legal_moves(&board).unwrap();

    let mut nodes = 0;
    for mov in moves {
        let mut board = board.clone();
        board.make(&mov).unwrap();

        let leaf_nodes = perft(&board, &move_generator, &hasher, &mut leaf_cache, depth - 1);
        println!("{} {}", mov, leaf_nodes);

        nodes += leaf_nodes;
    }

    println!("");
    println!("{}", nodes);

    if more_information {
        let end = Instant::now();
        let duration = end.duration_since(start);

        let nodes_per_second = nodes as f64 / duration.as_secs_f64();
        println!("Duration: {:?}", duration);
        println!("Nodes per second: {}", nodes_per_second);
    }

    Ok(())
}

fn perft(
    board: &Board,
    move_generator: &MoveGenerator,
    hasher: &ZobristHasher,
    cache: &mut HashMap<ZobristHash, usize>,
    depth: usize,
) -> usize {
    if depth == 0 {
        return 1;
    }

    let hash = board.hash ^ hasher.depth[depth];
    if let Some(hashed) = cache.get(&hash) {
        return *hashed;
    }

    let moves = move_generator.get_legal_moves(board).unwrap();
    if depth == 1 {
        cache.insert(hash, moves.len());
        return moves.len();
    }

    let mut nodes = 0;
    for mov in moves {
        let mut board = board.clone();
        board.make(&mov).unwrap();

        let next_nodes = perft(&board, move_generator, hasher, cache, depth - 1);
        nodes += next_nodes;
    }

    cache.insert(hash, nodes);
    nodes
}

fn table_generator_command() -> Result<(), Box<dyn std::error::Error>> {
    let mut output = String::new();
    lookup::generate_lookup_tables(&mut output)?;
    println!("{}", output);
    Ok(())
}
