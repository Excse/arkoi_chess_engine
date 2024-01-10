use std::io::{BufRead, Write};

use thiserror::Error;

use crate::{board::Board, move_generator::Move};

pub type Result<T> = std::result::Result<T, UCIError>;

#[derive(Debug, Error)]
pub enum UCIError {
    #[error("the command '{0}' is unknown")]
    UnknownCommand(String),
    #[error("{0}")]
    IOError(std::io::Error),
    #[error("you did not provide enough argument with the command '{0}'")]
    NotEnoughArguments(String),
    #[error("the given move '{0}' is not in a valid long algebraric notation")]
    InvalidMove(String),
    #[error("passed an invalid argument '{0}'")]
    InvalidArgument(&'static str),
}

impl From<std::io::Error> for UCIError {
    fn from(error: std::io::Error) -> Self {
        UCIError::IOError(error)
    }
}

#[derive(Debug)]
pub enum UCIOk {
    NewPosition(String, Vec<String>),
    IsReady,
    Go,
    Quit,
    None,
}

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

    // TODO: Debug using "info string"
    pub fn handle_command<R, W>(&mut self, reader: &mut R, writer: &mut W) -> Result<UCIOk>
    where
        R: BufRead,
        W: Write,
    {
        let input = UCI::read_input(reader)?;
        let tokens: Vec<&str> = input.split_whitespace().collect();
        let mut tokens = tokens.iter().peekable();

        let id = tokens
            .next()
            .ok_or(UCIError::NotEnoughArguments(input.clone()))?;
        match *id {
            "uci" => self.uci_received(writer),
            "debug" => self.debug_received(input.clone(), tokens),
            "isready" => self.isready_received(),
            "quit" => self.quit_received(),
            "position" => self.position_received(input.clone(), tokens),
            "go" => self.go_received(),
            _ => Err(UCIError::UnknownCommand(input)),
        }
    }

    // TODO: Debug using "info string"
    fn uci_received<W>(&self, writer: &mut W) -> Result<UCIOk>
    where
        W: Write,
    {
        self.send_id(writer)?;
        self.send_uciok(writer)?;

        Ok(UCIOk::None)
    }

    // TODO: Debug using "info string"
    fn debug_received(&mut self, command: String, mut tokens: TokenStream) -> Result<UCIOk> {
        let state = tokens.next().ok_or(UCIError::NotEnoughArguments(command))?;
        match *state {
            "on" => self.debug = true,
            "off" => self.debug = false,
            _ => {
                return Err(UCIError::InvalidArgument(
                    "debug can only be \"on\" or \"off\"",
                ))
            }
        }

        Ok(UCIOk::None)
    }

    // TODO: Debug using "info string"
    fn isready_received(&mut self) -> Result<UCIOk> {
        Ok(UCIOk::IsReady)
    }

    // TODO: Debug using "info string"
    fn quit_received(&mut self) -> Result<UCIOk> {
        Ok(UCIOk::Quit)
    }

    // TODO: Debug using "info string"
    fn position_received(&mut self, command: String, mut tokens: TokenStream) -> Result<UCIOk> {
        let board_fen: String;

        println!("{}", command);

        let variant = tokens
            .next()
            .ok_or(UCIError::NotEnoughArguments(command.clone()))?;
        match *variant {
            "fen" => {
                let items: Vec<_> = tokens.by_ref().take(6).cloned().collect();
                let fen_string = items.join(" ");
                board_fen = fen_string.to_string();
            }
            "startpos" => board_fen = Board::STARTPOS_FEN.to_string(),
            _ => {
                return Err(UCIError::InvalidArgument(
                    "the only valid variants are \"fen <fenstring>\" or \"startpos\"",
                ))
            }
        }

        let mut moves = Vec::new();
        match tokens.peek() {
            Some(&elem) if *elem == "moves" => tokens.next(),
            Some(..) => {
                return Err(UCIError::InvalidArgument(
                    "after the position variant only 'moves' can follow",
                ))
            }
            None => return Ok(UCIOk::NewPosition(board_fen, moves)),
        };

        while let Some(mov) = tokens.next() {
            moves.push(mov.to_string());
        }

        Ok(UCIOk::NewPosition(board_fen, moves))
    }

    // TODO: Add options to the UCIOk::Go
    // TODO: Debug using "info string"
    pub fn go_received(&mut self) -> Result<UCIOk> {
        Ok(UCIOk::Go)
    }

    // TODO: Debug using "info string"
    pub fn send_readyok<W>(&self, writer: &mut W) -> Result<UCIOk>
    where
        W: Write,
    {
        writeln!(writer, "readyok")?;

        Ok(UCIOk::None)
    }

    // TODO: Debug using "info string"
    pub fn send_id<W>(&self, writer: &mut W) -> Result<UCIOk>
    where
        W: Write,
    {
        writeln!(writer, "id name {}", self.name)?;
        writeln!(writer, "id author {}", self.author)?;

        Ok(UCIOk::None)
    }

    // TODO: Debug using "info string"
    pub fn send_uciok<W>(&self, writer: &mut W) -> Result<UCIOk>
    where
        W: Write,
    {
        writeln!(writer, "uciok")?;

        Ok(UCIOk::None)
    }

    pub fn send_bestmove<W>(&self, writer: &mut W, mov: &Move) -> Result<UCIOk>
    where
        W: Write,
    {
        writeln!(writer, "bestmove {}", mov)?;

        Ok(UCIOk::None)
    }

    fn read_input<R>(reader: &mut R) -> Result<String>
    where
        R: BufRead,
    {
        let mut buffer = String::new();
        reader.read_line(&mut buffer)?;
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::uci::UCIOk;

    use super::UCI;

    #[test]
    fn uci_command() {
        let mut uci = UCI::new("ace", "Excse");

        let mut reader = Cursor::new("   uci ");
        let mut writer = Vec::<u8>::new();

        let result = uci.handle_command(&mut reader, &mut writer);
        matches!(result, Ok(UCIOk::None));

        let output = String::from_utf8(writer).unwrap();
        let output: Vec<String> = output.lines().map(String::from).collect();

        assert_eq!(output.len(), 3);
        assert_eq!(output[0], "id name ace");
        assert_eq!(output[1], "id author Excse");
        assert_eq!(output[2], "uciok");
    }

    #[test]
    fn debug_command() {
        let mut uci = UCI::new("ace", "Excse");
        assert_eq!(uci.debug, false);

        let mut writer = Vec::<u8>::new();

        let mut reader = Cursor::new("   debug ");
        let result = uci.handle_command(&mut reader, &mut writer);
        assert!(result.is_err());

        let mut reader = Cursor::new("  debug   toggle   ");
        let result = uci.handle_command(&mut reader, &mut writer);
        assert!(result.is_err());

        let mut reader = Cursor::new(" debug   on ");
        let result = uci.handle_command(&mut reader, &mut writer);
        matches!(result, Ok(UCIOk::None));
        assert_eq!(uci.debug, true);

        let mut reader = Cursor::new("    debug   off  ");
        let result = uci.handle_command(&mut reader, &mut writer);
        matches!(result, Ok(UCIOk::None));
        assert_eq!(uci.debug, false);
    }

    #[test]
    fn isready_command() {
        let mut uci = UCI::new("ace", "Excse");
        assert_eq!(uci.debug, false);

        let mut reader = Cursor::new(" isready  ");
        let mut writer = Vec::<u8>::new();

        let result = uci.handle_command(&mut reader, &mut writer);
        matches!(result, Ok(UCIOk::IsReady));

        let result = uci.send_readyok(&mut writer);
        matches!(result, Ok(UCIOk::None));

        let output = String::from_utf8(writer).unwrap();
        let output: Vec<String> = output.lines().map(String::from).collect();

        assert_eq!(output.len(), 1);
        assert_eq!(output[0], "readyok");
    }

    #[test]
    fn quit_command() {
        let mut uci = UCI::new("ace", "Excse");

        let mut reader = Cursor::new("   quit ");
        let mut writer = Vec::<u8>::new();

        let result = uci.handle_command(&mut reader, &mut writer);
        matches!(result, Ok(UCIOk::Quit));
    }
}
