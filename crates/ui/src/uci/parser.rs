use crossbeam_channel::Sender;
use reedline::{Prompt, Reedline, Signal};

use base::board::Board;

use super::error::{InvalidArgument, NotEnoughArguments, UCIError, UnknownCommand};

type TokenStream<'a> = std::iter::Peekable<std::slice::Iter<'a, &'a str>>;

pub struct UCIParser<P: Prompt> {
    sender: Sender<UCICommand>,
    editor: Reedline,
    prompt: P,
}

impl<P: Prompt> UCIParser<P> {
    pub fn new(editor: Reedline, prompt: P, sender: Sender<UCICommand>) -> Self {
        Self {
            sender,
            prompt,
            editor,
        }
    }

    pub fn start(&mut self) -> Result<(), UCIError> {
        loop {
            let command = self.parse_command()?;

            match command {
                UCICommand::Quit => {
                    self.sender.send(command)?;
                    break;
                }
                _ => self.sender.send(command)?,
            };
        }

        Ok(())
    }

    fn parse_command(&mut self) -> Result<UCICommand, UCIError> {
        let input = match self.editor.read_line(&self.prompt) {
            Ok(Signal::Success(line)) => line,
            Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => return Ok(UCICommand::Quit),
            Err(error) => return Err(error.into()),
        };

        let tokens: Vec<&str> = input.split_whitespace().collect();
        let mut tokens = tokens.iter().peekable();

        let id = tokens
            .next()
            .ok_or(NotEnoughArguments::new(input.clone()))?;
        Ok(match *id {
            "uci" => UCICommand::UCI,
            "ucinewgame" => UCICommand::UCINewGame,
            "analyse" => UCICommand::Analyse,
            "debug" => {
                let result = DebugCommand::parse(&input, &mut tokens)?;
                UCICommand::Debug(result)
            }
            "isready" => UCICommand::IsReady,
            "quit" => UCICommand::Quit,
            "stop" => UCICommand::Stop,
            "position" => {
                let result = PositionCommand::parse(&input, &mut tokens)?;
                UCICommand::Position(result)
            }
            "go" => {
                let result = GoCommand::parse(&input, &mut tokens)?;
                UCICommand::Go(result)
            }
            "show" => UCICommand::Show,
            _ => return Err(UnknownCommand::new(input).into()),
        })
    }
}

#[derive(Debug)]
pub enum UCICommand {
    UCI,
    UCINewGame,
    Debug(DebugCommand),
    Position(PositionCommand),
    IsReady,
    Go(GoCommand),
    Stop,
    Quit,
    Show,
    Analyse,
}

#[derive(Default, Debug)]
pub struct DebugCommand {
    pub state: bool,
}

impl DebugCommand {
    pub fn parse(command: &String, tokens: &mut TokenStream) -> Result<Self, UCIError> {
        let state = match DebugToken::parse(command, tokens)? {
            DebugToken::On => true,
            DebugToken::Off => false,
        };

        Ok(Self { state })
    }
}

#[derive(Debug)]
pub enum DebugToken {
    On,
    Off,
}

impl DebugToken {
    pub fn parse(command: &String, tokens: &mut TokenStream) -> Result<DebugToken, UCIError> {
        let token = tokens
            .next()
            .ok_or(NotEnoughArguments::new(command.clone()))?;

        match *token {
            "on" => Ok(DebugToken::On),
            "off" => Ok(DebugToken::Off),
            _ => Err(InvalidArgument::new("{} is not a valid argument").into()),
        }
    }
}

#[derive(Default, Debug)]
pub struct PositionCommand {
    pub fen: String,
    pub moves: Vec<String>,
}

impl PositionCommand {
    pub fn parse(command: &String, tokens: &mut TokenStream) -> Result<Self, UCIError> {
        let mut result = Self::default();

        let fen = match PositionToken::parse(command, tokens)? {
            PositionToken::Fen(fen) => fen,
            _ => return Err(InvalidArgument::new("Expected 'fen' or 'startpos'").into()),
        };
        result.fen = fen;

        let moves = match PositionToken::parse(command, tokens) {
            Ok(PositionToken::Moves(moves)) => moves,
            Err(UCIError::NotEnoughArguments(_)) => Vec::new(),
            Err(error) => return Err(error),
            _ => Vec::new(),
        };
        result.moves = moves;

        Ok(result)
    }
}

#[derive(Debug)]
pub enum PositionToken {
    Fen(String),
    Moves(Vec<String>),
}

impl PositionToken {
    pub fn is_token(input: &str) -> bool {
        match input {
            "fen" => true,
            "startpos" => true,
            "moves" => true,
            _ => false,
        }
    }

    pub fn parse(command: &String, tokens: &mut TokenStream) -> Result<PositionToken, UCIError> {
        let token = tokens
            .next()
            .ok_or(NotEnoughArguments::new(command.clone()))?;

        match *token {
            "fen" => {
                let items: Vec<_> = tokens.by_ref().take(6).cloned().collect();
                let fen_string = items.join(" ");
                Ok(PositionToken::Fen(fen_string))
            }
            "startpos" => Ok(PositionToken::Fen(Board::STARTPOS_FEN.to_string())),
            "moves" => {
                let moves = tokens
                    .by_ref()
                    .take_while(|&token| !Self::is_token(token))
                    .map(|token| token.to_string())
                    .collect::<Vec<String>>();
                Ok(PositionToken::Moves(moves))
            }
            _ => Err(InvalidArgument::new(format!("'{}' is not a valid argument", token)).into()),
        }
    }
}

#[derive(Default, Debug)]
pub struct GoCommand {
    pub search_moves: Vec<String>,
    pub ponder: bool,
    pub white_time: Option<u128>,
    pub black_time: Option<u128>,
    pub white_increment: Option<u128>,
    pub black_increment: Option<u128>,
    pub moves_to_go: Option<usize>,
    pub depth: Option<u8>,
    pub nodes: Option<usize>,
    pub move_time: Option<u128>,
    pub infinite: bool,
}

impl GoCommand {
    pub fn parse(command: &String, tokens: &mut TokenStream) -> Result<Self, UCIError> {
        let mut result = Self::default();

        while tokens.peek().is_some() {
            let token = GoToken::parse(command, tokens)?;
            match token {
                GoToken::SearchMoves(moves) => result.search_moves = moves,
                GoToken::Ponder => result.ponder = true,
                GoToken::WhiteTime(time) => result.white_time = Some(time),
                GoToken::BlackTime(time) => result.black_time = Some(time),
                GoToken::WhiteIncrement(inc) => result.white_increment = Some(inc),
                GoToken::BlackIncrement(inc) => result.black_increment = Some(inc),
                GoToken::MovesToGo(moves) => result.moves_to_go = Some(moves),
                GoToken::Depth(depth) => result.depth = Some(depth),
                GoToken::Nodes(nodes) => result.nodes = Some(nodes),
                GoToken::MoveTime(time) => result.move_time = Some(time),
                GoToken::Infinite => result.infinite = true,
            }
        }

        Ok(result)
    }
}

pub enum GoToken {
    SearchMoves(Vec<String>),
    Ponder,
    WhiteTime(u128),
    BlackTime(u128),
    WhiteIncrement(u128),
    BlackIncrement(u128),
    MovesToGo(usize),
    Depth(u8),
    Nodes(usize),
    MoveTime(u128),
    Infinite,
}

impl GoToken {
    pub fn is_token(input: &str) -> bool {
        match input {
            "searchmoves" => true,
            "ponder" => true,
            "wtime" => true,
            "btime" => true,
            "winc" => true,
            "binc" => true,
            "movestogo" => true,
            "depth" => true,
            "nodes" => true,
            "movetime" => true,
            "infinite" => true,
            _ => false,
        }
    }

    pub fn parse(command: &String, tokens: &mut TokenStream) -> Result<GoToken, UCIError> {
        let token = tokens
            .next()
            .ok_or(NotEnoughArguments::new(command.clone()))?;

        match *token {
            "ponder" => return Ok(GoToken::Ponder),
            "searchmoves" => {
                let moves = tokens
                    .by_ref()
                    .take_while(|&token| !Self::is_token(token))
                    .map(|token| token.to_string())
                    .collect::<Vec<String>>();
                return Ok(GoToken::SearchMoves(moves));
            }
            "wtime" => {
                let white_time = tokens
                    .next()
                    .ok_or(NotEnoughArguments::new(command.clone()))?;
                let white_time = white_time.parse::<u128>()?;
                return Ok(GoToken::WhiteTime(white_time));
            }
            "btime" => {
                let black_time = tokens
                    .next()
                    .ok_or(NotEnoughArguments::new(command.clone()))?;
                let black_time = black_time.parse::<u128>()?;
                return Ok(GoToken::BlackTime(black_time));
            }
            "winc" => {
                let white_increment = tokens
                    .next()
                    .ok_or(NotEnoughArguments::new(command.clone()))?;
                let white_increment = white_increment.parse::<u128>()?;
                return Ok(GoToken::WhiteIncrement(white_increment));
            }
            "binc" => {
                let black_increment = tokens
                    .next()
                    .ok_or(NotEnoughArguments::new(command.clone()))?;
                let black_increment = black_increment.parse::<u128>()?;
                return Ok(GoToken::BlackIncrement(black_increment));
            }
            "movestogo" => {
                let moves_to_go = tokens
                    .next()
                    .ok_or(NotEnoughArguments::new(command.clone()))?;
                let moves_to_go = moves_to_go.parse::<usize>()?;
                return Ok(GoToken::MovesToGo(moves_to_go));
            }
            "depth" => {
                let depth = tokens
                    .next()
                    .ok_or(NotEnoughArguments::new(command.clone()))?;
                let depth = depth.parse::<u8>()?;
                return Ok(GoToken::Depth(depth));
            }
            "nodes" => {
                let nodes = tokens
                    .next()
                    .ok_or(NotEnoughArguments::new(command.clone()))?;
                let nodes = nodes.parse::<usize>()?;
                return Ok(GoToken::Nodes(nodes));
            }
            "movetime" => {
                let movetime = tokens
                    .next()
                    .ok_or(NotEnoughArguments::new(command.clone()))?;
                let movetime = movetime.parse::<u128>()?;
                return Ok(GoToken::MoveTime(movetime));
            }
            "infinite" => return Ok(GoToken::Infinite),
            _ => {
                return Err(
                    InvalidArgument::new(format!("'{}' is not a valid argument", token)).into(),
                )
            }
        }
    }
}
