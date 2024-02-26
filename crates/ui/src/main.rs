use std::thread;

use reedline::{ExternalPrinter, Reedline};
use uci::{controller::UCIController, parser::UCIParser, prompt::CustomPrompt};

mod test;
mod uci;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (sender, receiver) = crossbeam_channel::unbounded();
    let printer = ExternalPrinter::<String>::default();
    let controller_printer = printer.clone();

    // TODO: Remove unwrap
    let controller = thread::spawn(move || {
        let mut uci_controller = UCIController::new(controller_printer, receiver);
        uci_controller.start().unwrap();
    });

    // TODO: Remove unwrap
    let parser = thread::spawn(move || {
        let editor = Reedline::create().with_external_printer(printer);
        let prompt = CustomPrompt::default();

        let mut uci_input = UCIParser::new(editor, prompt, sender);
        uci_input.start().unwrap();
    });

    parser.join().expect("Couldnt join the parser thread");
    controller
        .join()
        .expect("Couldnt join the controller thread");

    Ok(())
}

// fn perft_command(
//     depth: u8,
//     fen: String,
//     cache_size: usize,
//     moves: Vec<String>,
//     more_information: bool,
//     hashed: bool,
//     divide: bool,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let mut rand = rand::thread_rng();
//     let hasher = ZobristHasher::new(&mut rand);

//     let mut board = Board::from_str(&fen, hasher)?;
//     board.make_moves(&moves)?;

//     let mut cache = HashTable::size(cache_size);

//     let start = Instant::now();

//     let nodes = if divide {
//         if hashed {
//             perft::divide::<true>(&mut board, &mut cache, depth)
//         } else {
//             perft::divide::<false>(&mut board, &mut cache, depth)
//         }
//     } else {
//         if hashed {
//             perft::perft_normal::<true>(&mut board, &mut cache, depth)
//         } else {
//             perft::perft_normal::<false>(&mut board, &mut cache, depth)
//         }
//     };

//     if more_information {
//         let end = Instant::now();
//         let duration = end.duration_since(start);

//         let nodes_per_second = nodes as f64 / duration.as_secs_f64();
//         println!("Duration: {:?}", duration);
//         println!("Nodes per second: {}", nodes_per_second);
//     }

//     Ok(())
// }
