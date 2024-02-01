use std::{
    io::{stdin, stdout},
    time::Instant,
};

use clap::{Parser, Subcommand};
use rand::seq::SliceRandom;

use board::{zobrist::ZobristHasher, Board};
use move_generator::{error::MoveGeneratorError, mov::Move, MoveGenerator};
use uci::{Command, UCI};

mod bitboard;
mod board;
mod lookup;
mod move_generator;
mod uci;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand)]
enum CliCommand {
    UCI,
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
        CliCommand::UCI => uci_command(),
        CliCommand::Perft {
            depth,
            fen,
            moves,
            more_information,
        } => perft_command(depth, fen, moves, more_information),
        CliCommand::TableGenerator => table_generator_command(),
    }
}

fn uci_command() -> Result<(), Box<dyn std::error::Error>> {
    let name = format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    let author = env!("CARGO_PKG_AUTHORS");
    let mut uci = UCI::new(name, author);

    let mut rng = rand::thread_rng();
    let mut reader = stdin().lock();
    let mut writer = stdout();

    let move_generator = MoveGenerator::default();
    let hasher = ZobristHasher::new();
    let mut board = Board::default(&hasher);
    loop {
        let result = uci.receive_command(&mut reader, &mut writer);
        match result {
            Ok(Command::NewPosition(fen, moves)) => {
                board = Board::from_str(&fen, &hasher)?;

                for mov_str in moves {
                    let mov = Move::parse(mov_str, board.active, &board)?;
                    board.play(board.active, &mov)?;
                    board.swap_active();
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
                println!("FEN {}:", board.to_fen());

                let moves = move_generator.get_legal_moves(&board)?;
                println!("Moves {}:", moves.len());
                print!(" - ");
                for mov in moves {
                    print!("{}, ", mov);
                }
                println!();

                if let Some(en_passant) = board.en_passant {
                    println!(
                        "En passant: Capture {} and move to {}",
                        en_passant.to_capture, en_passant.to_move
                    );
                }
            }
            Ok(Command::Go) => {
                let moves = move_generator.get_legal_moves(&board)?;
                let mov = moves.choose(&mut rng).expect("There should be a move");
                uci.send_bestmove(&mut writer, mov)?;
                board.play(board.active, mov)?;
                board.swap_active();
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
    let hasher = ZobristHasher::new();
    let mut board = Board::from_str(&fen, &hasher)?;

    for mov_str in moves {
        let mov = Move::parse(mov_str, board.active, &board)?;
        board.play(board.active, &mov)?;
        board.swap_active();
    }

    let move_generator = MoveGenerator::default();

    let start = Instant::now();

    let nodes = perft(&board, &move_generator, depth, true)?;

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
    depth: usize,
    print_moves: bool,
) -> Result<usize, MoveGeneratorError> {
    if depth == 0 {
        return Ok(1);
    }

    let mut nodes = 0;

    let moves = move_generator.get_legal_moves(board).unwrap();
    for mov in moves {
        let leaf_nodes = if depth > 1 {
            let mut board = board.clone();
            board.play(board.active, &mov)?;
            board.swap_active();

            perft(&board, move_generator, depth - 1, false).unwrap()
        } else {
            1
        };
        nodes += leaf_nodes;

        if print_moves {
            println!("{} {}", mov, leaf_nodes);
        }
    }

    Ok(nodes)
}

fn table_generator_command() -> Result<(), Box<dyn std::error::Error>> {
    lookup::generate_lookup_tables();
    Ok(())
}
