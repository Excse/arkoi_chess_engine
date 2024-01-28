use std::{
    io::{stdin, stdout},
    str::FromStr,
};

use clap::{Parser, Subcommand};
use rand::seq::SliceRandom;

use board::Board;
use move_generator::{error::MoveGeneratorError, Move, MoveGenerator};
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
        depth: usize,
        fen: String,
        #[clap(value_parser, num_args = 0.., value_delimiter = ' ')]
        moves: Vec<String>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        CliCommand::UCI => uci_command(),
        CliCommand::Perft { depth, fen, moves } => perft_command(depth, fen, moves),
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
    let mut board = Board::default();
    loop {
        let result = uci.receive_command(&mut reader, &mut writer);
        match result {
            Ok(Command::NewPosition(fen, moves)) => {
                board = Board::from_str(&fen)?;

                for mov_str in moves {
                    let mov = Move::parse(mov_str, &board)?;
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
                let moves = move_generator.get_legal_moves(&board)?;
                println!("Moves: {}", moves.len());
                for mov in moves {
                    println!(" - {}", mov);
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
) -> Result<(), Box<dyn std::error::Error>> {
    let mut board = Board::from_str(&fen)?;

    for mov_str in moves {
        let mov = Move::parse(mov_str, &board)?;
        board.play(board.active, &mov)?;
        board.swap_active();
    }

    let move_generator = MoveGenerator::default();

    let result = perft(&board, &move_generator, depth, true)?;
    println!("");
    println!("{}", result);

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

    let moves = move_generator.get_legal_moves(board)?;

    let mut result = 0;
    for mov in moves {
        let mut board = board.clone();
        board.play(board.active, &mov)?;
        board.swap_active();

        let next_perft = perft(&board, move_generator, depth - 1, false)?;
        result += next_perft;

        if print_moves {
            println!("{} {}", mov, next_perft);
        }
    }

    Ok(result)
}
