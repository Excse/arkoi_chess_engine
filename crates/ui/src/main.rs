use std::thread;

use uci::{
    controller::UCIController,
    parser::{UCICommand, UCIParser},
};

mod uci;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (sender, receiver) = crossbeam_channel::unbounded();

    let controller_receiver = receiver.clone();

    // TODO: Remove unwrap
    let handler = thread::spawn(move || {
        let mut uci_controller = UCIController::new(controller_receiver);
        uci_controller.start().unwrap();
    });

    let parser_sender = sender.clone();

    let mut uci_input = UCIParser::new(parser_sender);
    uci_input.start()?;

    sender.send(UCICommand::Quit)?;
    // TODO: Remove unwrap
    handler.join().unwrap();

    Ok(())
}
