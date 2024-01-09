use std::io::{BufRead, Write};

use thiserror::Error;

pub type Result<T> = std::result::Result<T, UCIError>;

#[derive(Debug, Error)]
pub enum UCIError {
    #[error("the command '{0}' is unknown")]
    UnknownCommand(String),
    #[error("{0}")]
    IOError(std::io::Error),
    #[error("there are not enough arguments for the given command")]
    // TODO: Add the given command
    NotEnoughArguments,
    #[error("passed an invalid argument '{0}'")]
    InvalidArgument(&'static str),
}

impl From<std::io::Error> for UCIError {
    fn from(error: std::io::Error) -> Self {
        UCIError::IOError(error)
    }
}

pub struct UCI {
    name: String,
    author: String,
    debug: bool,
    pub running: bool,
}

impl UCI {
    pub fn new(name: impl Into<String>, author: impl Into<String>) -> UCI {
        UCI {
            name: name.into(),
            author: author.into(),
            debug: false,
            running: true,
        }
    }

    // TODO: Debug using "info string"
    pub fn handle_command<R, W>(&mut self, reader: &mut R, writer: &mut W) -> Result<()>
    where
        R: BufRead,
        W: Write,
    {
        let input = UCI::read_input(reader)?;
        let tokens: Vec<&str> = input.split_whitespace().collect();

        match tokens.as_slice() {
            ["uci"] => self.uci_received(writer),
            ["debug", tokens @ ..] => self.debug_received(tokens),
            ["isready"] => self.isready_received(writer),
            ["quit"] => self.quit_received(),
            _ => Err(UCIError::UnknownCommand(input)),
        }
    }

    // TODO: Debug using "info string"
    fn uci_received<W>(&self, writer: &mut W) -> Result<()>
    where
        W: Write,
    {
        self.send_id(writer)?;
        self.send_uciok(writer)?;
        Ok(())
    }

    // TODO: Debug using "info string"
    fn debug_received(&mut self, tokens: &[&str]) -> Result<()> {
        let state = tokens.get(0).ok_or(UCIError::NotEnoughArguments)?;

        match *state {
            "on" => self.debug = true,
            "off" => self.debug = false,
            _ => return Err(UCIError::InvalidArgument("Debug can only be on or off.")),
        }

        Ok(())
    }

    // TODO: Debug using "info string"
    // TODO: Handle differently, maybe blocking until ready
    fn isready_received<W>(&mut self, writer: &mut W) -> Result<()>
    where
        W: Write,
    {
        self.send_readyok(writer)?;
        Ok(())
    }

    // TODO: Debug using "info string"
    fn quit_received(&mut self) -> Result<()> {
        self.running = false;
        Ok(())
    }

    // TODO: Debug using "info string"
    fn send_readyok<W>(&self, writer: &mut W) -> Result<()>
    where
        W: Write,
    {
        writeln!(writer, "readyok")?;
        Ok(())
    }

    // TODO: Debug using "info string"
    fn send_id<W>(&self, writer: &mut W) -> Result<()>
    where
        W: Write,
    {
        writeln!(writer, "id name {}", self.name)?;
        writeln!(writer, "id author {}", self.author)?;
        Ok(())
    }

    // TODO: Debug using "info string"
    fn send_uciok<W>(&self, writer: &mut W) -> Result<()>
    where
        W: Write,
    {
        writeln!(writer, "uciok")?;
        Ok(())
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

    use super::UCI;

    #[test]
    fn uci_command() {
        let mut uci = UCI::new("ace", "Excse");

        let mut reader = Cursor::new("   uci ");
        let mut writer = Vec::<u8>::new();

        let result = uci.handle_command(&mut reader, &mut writer);
        assert!(result.is_ok());

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
        assert!(result.is_ok());
        assert_eq!(uci.debug, true);

        let mut reader = Cursor::new("    debug   off  ");
        let result = uci.handle_command(&mut reader, &mut writer);
        assert!(result.is_ok());
        assert_eq!(uci.debug, false);
    }

    #[test]
    fn isready_command() {
        let mut uci = UCI::new("ace", "Excse");
        assert_eq!(uci.debug, false);

        let mut reader = Cursor::new(" isready  ");
        let mut writer = Vec::<u8>::new();

        let result = uci.handle_command(&mut reader, &mut writer);
        assert!(result.is_ok());

        let output = String::from_utf8(writer).unwrap();
        let output: Vec<String> = output.lines().map(String::from).collect();

        assert_eq!(output.len(), 1);
        assert_eq!(output[0], "readyok");
    }

    #[test]
    fn quit_command() {
        let mut uci = UCI::new("ace", "Excse");
        assert_eq!(uci.running, true);

        let mut reader = Cursor::new("   quit ");
        let mut writer = Vec::<u8>::new();

        let result = uci.handle_command(&mut reader, &mut writer);
        assert!(result.is_ok());
        assert_eq!(uci.running, false);
    }
}
