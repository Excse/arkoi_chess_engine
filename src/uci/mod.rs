pub mod commands;
pub mod error;
mod tests;

use std::io::{BufRead, Write};

use self::{
    commands::{Command, DebugCommand, GoCommand, PositionCommand},
    error::{NotEnoughArguments, UCIError, UnknownCommand},
};
use crate::move_generator::mov::Move;

pub struct UCI {
    name: String,
    author: String,
    debug: bool,
}

type TokenStream<'a> = std::iter::Peekable<std::slice::Iter<'a, &'a str>>;

impl UCI {
    pub fn new(name: impl Into<String>, author: impl Into<String>) -> UCI {
        UCI {
            name: name.into(),
            author: author.into(),
            debug: false,
        }
    }

    pub fn receive_command(
        &mut self,
        reader: &mut impl BufRead,
        writer: &mut impl Write,
    ) -> Result<Command, UCIError> {
        let input = UCI::read_input(reader)?;
        let tokens: Vec<&str> = input.split_whitespace().collect();
        let mut tokens = tokens.iter().peekable();

        let id = tokens
            .next()
            .ok_or(NotEnoughArguments::new(input.clone()))?;
        match *id {
            "uci" => self.received_uci(writer),
            "debug" => self.received_debug(writer, &input, &mut tokens),
            "isready" => self.received_isready(writer),
            "quit" => self.received_quit(writer),
            "position" => self.received_position(writer, &input, &mut tokens),
            "show" => self.received_show(writer),
            "go" => self.received_go(writer, &input, &mut tokens),
            _ => Err(UnknownCommand::new(input).into()),
        }
    }

    fn received_uci(&self, writer: &mut impl Write) -> Result<Command, UCIError> {
        self.send_debug(writer, "Command Received: uci")?;

        self.send_id(writer)?;
        self.send_uciok(writer)?;

        Ok(Command::None)
    }

    fn received_debug(
        &mut self,
        writer: &mut impl Write,
        command: &String,
        tokens: &mut TokenStream,
    ) -> Result<Command, UCIError> {
        let result = DebugCommand::parse(command, tokens)?;
        self.debug = result.state;

        self.send_debug(writer, format!("Command Received: {:?}", result))?;

        Ok(Command::None)
    }

    fn received_isready(&mut self, writer: &mut impl Write) -> Result<Command, UCIError> {
        self.send_debug(writer, "Command Received: isready")?;

        Ok(Command::IsReady)
    }

    fn received_quit(&mut self, writer: &mut impl Write) -> Result<Command, UCIError> {
        self.send_debug(writer, "Command Received: quit")?;

        Ok(Command::Quit)
    }

    fn received_position(
        &mut self,
        writer: &mut impl Write,
        command: &String,
        tokens: &mut TokenStream,
    ) -> Result<Command, UCIError> {
        let result = PositionCommand::parse(command, tokens)?;

        self.send_debug(writer, format!("Command Received: {:?}", result))?;

        Ok(Command::Position(result))
    }

    pub fn received_go(
        &mut self,
        writer: &mut impl Write,
        command: &String,
        tokens: &mut TokenStream,
    ) -> Result<Command, UCIError> {
        let result = GoCommand::parse(command, tokens)?;

        self.send_debug(writer, format!("Command Received: {:?}", result))?;

        Ok(Command::Go(result))
    }

    pub fn received_show(&mut self, writer: &mut impl Write) -> Result<Command, UCIError> {
        self.send_debug(writer, "Command Received: show")?;

        Ok(Command::Show)
    }

    pub fn send_readyok(&self, writer: &mut impl Write) -> Result<Command, UCIError> {
        self.send_debug(writer, "Command Send: readyok")?;

        writeln!(writer, "readyok")?;

        Ok(Command::None)
    }

    pub fn send_id(&self, writer: &mut impl Write) -> Result<Command, UCIError> {
        self.send_debug(writer, "Command Send: id")?;

        writeln!(writer, "id name {}", self.name)?;
        writeln!(writer, "id author {}", self.author)?;

        Ok(Command::None)
    }

    pub fn send_uciok(&self, writer: &mut impl Write) -> Result<Command, UCIError> {
        self.send_debug(writer, "Command Send: uciok")?;

        writeln!(writer, "uciok")?;

        Ok(Command::None)
    }

    pub fn send_bestmove(&self, writer: &mut impl Write, mov: &Move) -> Result<Command, UCIError> {
        self.send_debug(writer, format!("Command Send: bestmove {}", mov))?;

        writeln!(writer, "bestmove {}", mov)?;

        Ok(Command::None)
    }

    pub fn send_debug(
        &self,
        writer: &mut impl Write,
        message: impl Into<String>,
    ) -> Result<Command, UCIError> {
        if self.debug {
            writeln!(writer, "info string {}", message.into())?;
        }

        Ok(Command::None)
    }

    fn read_input(reader: &mut impl BufRead) -> Result<String, UCIError> {
        let mut buffer = String::new();
        reader.read_line(&mut buffer)?;
        Ok(buffer)
    }
}
