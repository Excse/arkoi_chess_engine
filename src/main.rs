#![allow(dead_code)]

use std::{
    io::{stdin, stdout},
    str::FromStr,
};

use rand::seq::SliceRandom;

use board::Board;
use move_generator::MoveGenerator;
use uci::{UCIError, UCIOk, UCI};

use crate::board::{Color, Piece};

mod bitboard;
mod board;
mod move_generator;
mod tables;
mod uci;

fn main() {
    let name = format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    let author = env!("CARGO_PKG_AUTHORS");
    let mut uci = UCI::new(name, author);

    let mut rng = rand::thread_rng();
    let mut reader = stdin().lock();
    let mut writer = stdout();

    let mut board = Board::default();
    let mut move_generator = MoveGenerator::new(&board);
    loop {
        let result = uci.handle_command(&mut reader, &mut writer);
        match result {
            Ok(UCIOk::NewPosition(fen, moves)) => {
                board = Board::from_str(&fen).unwrap();
                for mov in moves {
                    board.play_active(&mov);
                    board.swap_active();
                }

                println!("{:?}", board.active);
                println!("Occupied:\n{}", board.get_occupied());
                println!("Active:\n{}", board.get_active());
                println!(
                    "Black Pawns:\n{}",
                    board.get_piece_board(Color::Black, Piece::Pawn)
                );
                println!(
                    "White Pawns:\n{}",
                    board.get_piece_board(Color::White, Piece::Pawn)
                );

                move_generator = MoveGenerator::new(&board);
            }
            Ok(UCIOk::IsReady) => {
                uci.send_readyok(&mut writer).unwrap();
            }
            Ok(UCIOk::Quit) => {
                break;
            }
            Ok(UCIOk::Play(_mov)) => {}
            Ok(UCIOk::Go) => {
                let moves = move_generator.get_pseudo_moves().unwrap();
                let mov = moves.choose(&mut rng).unwrap();
                uci.send_bestmove(&mut writer, mov).unwrap();
            }
            Ok(UCIOk::None) => {}
            Err(error) => eprintln!("{:?}", error),
        }
    }
}
