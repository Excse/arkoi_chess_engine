use std::{
    io::{stdin, stdout},
    path::Path,
    time::Instant,
};

use clap::{Parser, Subcommand};

use board::{color::Color, zobrist::ZobristHasher, Board};
use move_generator::mov::Move;
use search::{minimax, transposition::TranspositionEntry};
use uci::{Command, UCI};

use crate::search::{evaluate, transposition::TranspositionTable};

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
        depth: u8,
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

                let move_state = board.get_legal_moves()?;
                println!("Moves {}:", move_state.moves.len());
                print!(" - ");
                for mov in move_state.moves {
                    print!("{}, ", mov);
                }
                println!();
                println!("Checkmate: {}", move_state.is_checkmate);
                println!("Stalemate: {}", move_state.is_stalemate);
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
    depth: u8,
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

    // TODO: Fixed 4GB, add a parameter to the command
    let mut cache = TranspositionTable::new(4 * 1024 * 1024 * 1024);

    let start = Instant::now();

    let move_state = board.get_legal_moves().unwrap();

    let mut nodes = 0;
    for mov in move_state.moves {
        let mut board = board.clone();
        board.make(&mov).unwrap();

        let leaf_nodes = perft(&board, &hasher, &mut cache, depth - 1);
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

fn perft(board: &Board, hasher: &ZobristHasher, cache: &mut TranspositionTable, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let hash = board.hash ^ hasher.depth[depth as usize];
    if let Some(hashed) = cache.probe(hash) {
        return hashed.nodes;
    }

    let move_state = board.get_legal_moves().unwrap();
    if move_state.is_stalemate || move_state.is_checkmate {
        return 0;
    }

    if depth == 1 {
        let moves = move_state.moves.len() as u64;
        cache.store(TranspositionEntry::new(hash, depth, moves));
        return moves;
    }

    let mut nodes = 0;
    for mov in move_state.moves {
        let mut board = board.clone();
        board.make(&mov).unwrap();

        let next_nodes = perft(&board, hasher, cache, depth - 1);
        nodes += next_nodes;
    }

    cache.store(TranspositionEntry::new(hash, depth, nodes));
    nodes
}

fn table_generator_command() -> Result<(), Box<dyn std::error::Error>> {
    let mut output = String::new();
    lookup::generate_lookup_tables(&mut output)?;

    let path = Path::new("./src/lookup/tables.rs");
    std::fs::write(path, output)?;

    Ok(())
}
