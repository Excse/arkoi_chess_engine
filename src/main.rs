#![allow(dead_code)]

use std::str::FromStr;

use board::Board;
use move_generator::MoveGenerator;

mod bitboard;
mod board;
mod move_generator;
mod tables;
mod uci;

fn main() {
    // let name = format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
    // let author = env!("CARGO_PKG_AUTHORS");
    // let mut uci = UCI::new(name, author);

    // let mut reader = stdin().lock();
    // let mut writer = stdout();

    let board =
        Board::from_str("rnbqkbnr/ppp1pp1p/8/3p4/2P3p1/5P1P/PP1PP1P1/RNBQKBNR w KQkq - 0 4")
            .unwrap();
    // let board = Board::default();
    let move_generator = MoveGenerator::new(board);
    let pseudo_moves = move_generator.get_pseudo_moves().unwrap();
    for pseudo_move in pseudo_moves {
        println!("{}", pseudo_move);
    }
    // loop {
    //     if !uci.running {
    //         break;
    //     }

    //     let result = uci.handle_command(&mut reader, &mut writer);
    //     match result {
    //         Ok(..) => {}
    //         Err(UCIError::UnknownCommand(command)) => println!("Unknown command {}!", command),
    //         Err(error) => panic!("{:?}", error),
    //     }
    // }
}
