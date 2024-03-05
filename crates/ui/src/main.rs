use std::thread;

<<<<<<< Updated upstream
use uci::{
    controller::UCIController,
    parser::{UCICommand, UCIParser},
};
=======
use uci::{controller::UCIController, parser::UCIParser, prompt::CustomPrompt};
>>>>>>> Stashed changes

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
