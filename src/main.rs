#![allow(dead_code)]

use std::{
    io::{stdin, stdout},
    str::FromStr,
};

use rand::seq::SliceRandom;

use board::Board;
use move_generator::MoveGenerator;
use uci::{UCIOk, UCI};

use crate::move_generator::Move;

mod bitboard;
mod board;
mod move_generator;
mod tables;
mod uci;

// TODO: Handle unwrap
fn main() {
    let name = format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    let author = env!("CARGO_PKG_AUTHORS");
    let mut uci = UCI::new(name, author);

    let mut rng = rand::thread_rng();
    let mut reader = stdin().lock();
    let mut writer = stdout();

    let move_generator = MoveGenerator::default();
    let mut board = Board::default();
    loop {
        let result = uci.handle_command(&mut reader, &mut writer);
        match result {
            Ok(UCIOk::NewPosition(fen, moves)) => {
                board = Board::from_str(&fen).unwrap();
                for mov_str in moves {
                    let mov = Move::parse(mov_str, &board).unwrap();
                    board.play_active(&mov).unwrap();
                    board.swap_active();
                }
            }
            Ok(UCIOk::IsReady) => {
                uci.send_readyok(&mut writer).unwrap();
            }
            Ok(UCIOk::Quit) => {
                break;
            }
            Ok(UCIOk::Go) => {
                let moves = move_generator.get_pseudo_moves(&board).unwrap();
                let mov = moves.choose(&mut rng).unwrap();
                uci.send_bestmove(&mut writer, mov).unwrap();
                board.play_active(mov).unwrap();
                board.swap_active();
            }
            Ok(UCIOk::None) => {}
            Err(error) => eprintln!("{:?}", error),
        }
    }
}
