use std::{
    io::{stdin, stdout},
    str::FromStr,
};

use rand::seq::SliceRandom;

use board::Board;
use move_generator::{Move, MoveGenerator};
use uci::{Command, UCI};

mod bitboard;
mod board;
mod lookup;
mod move_generator;
mod uci;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
                    board.play_active(&mov)?;
                    board.swap_active();
                }
            }
            Ok(Command::IsReady) => {
                uci.send_readyok(&mut writer)?;
            }
            Ok(Command::Quit) => {
                return Ok(());
            }
            Ok(Command::Go) => {
                let moves = move_generator.get_legal_moves(&board)?;
                for mov in &moves {
                    println!("{}", mov);
                }

                let mov = moves.choose(&mut rng).expect("There should be a move");
                uci.send_bestmove(&mut writer, mov)?;
                board.play_active(mov)?;
                board.swap_active();
            }
            Ok(Command::None) => {}
            Err(error) => eprintln!("{:?}", error),
        }
    }
}
