use std::thread;

use uci::{
    controller::UCIController,
    parser::{UCICommand, UCIParser},
};

mod uci;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (sender, receiver) = crossbeam_channel::unbounded();

    let mut controller = UCIController::new(receiver.clone())?;
    let handler = thread::spawn(move || controller.start());

    let mut uci_input = UCIParser::new(sender.clone());
    uci_input.start()?;

    sender.send(UCICommand::Quit)?;
    handler
        .join()
        .expect("Couldn't join the controller thread")?;

    Ok(())
}
