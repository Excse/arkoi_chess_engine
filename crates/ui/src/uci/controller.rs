use crossbeam_channel::Receiver;
use reedline::ExternalPrinter;

use base::{board::Board, r#move::Move, zobrist::ZobristHasher};
use engine::{
    evaluation::evaluate,
    generator::MoveGenerator,
    hashtable::TranspositionTable,
    search::{search, SearchInfo, MAX_DEPTH},
};

use super::{
    error::UCIError,
    parser::{DebugCommand, GoCommand, PositionCommand, UCICommand}, printer::Printer,
};

// Around 32MB
pub const DEFAULT_CACHE_SIZE: usize = 2 << 25;
pub const LICHESS_ANALYSIS_BASE: &str = "https://lichess.org/analysis";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
pub const NAME: &str = env!("CARGO_PKG_NAME");

pub struct UCIController {
    receiver: Receiver<UCICommand>,
    cache: TranspositionTable,
    printer: ExternalPrinter<String>,
    hasher: ZobristHasher,
    board: Board,
    debug: bool,
}

impl UCIController {
    pub fn new(printer: ExternalPrinter<String>, receiver: Receiver<UCICommand>) -> Self {
        let cache = TranspositionTable::size(DEFAULT_CACHE_SIZE);

        let mut rand = rand::thread_rng();
        let hasher = ZobristHasher::new(&mut rand);

        let board = Board::default(hasher.clone());

        Self {
            receiver,
            cache,
            hasher,
            printer,
            board,
            debug: false,
        }
    }

    pub fn start(&mut self) -> Result<(), UCIError> {
        loop {
            let command = self.receiver.recv()?;
            match command {
                UCICommand::Quit => {
                    self.handle_uci(command)?;
                    break;
                }
                command => self.handle_uci(command)?,
            }
        }

        Ok(())
    }

    fn handle_uci(&mut self, command: UCICommand) -> Result<(), UCIError> {
        self.send_debug(format!("Received Command: {:?}", command))?;

        match command {
            UCICommand::Position(command) => self.uci_position(command),
            UCICommand::Debug(command) => self.received_debug(command),
            UCICommand::UCINewGame => self.received_uci_new_game(),
            UCICommand::Go(command) => self.received_go(command),
            UCICommand::IsReady => self.received_isready(),
            UCICommand::Analyse => self.received_analyse(),
            UCICommand::Stop => self.received_stop(),
            UCICommand::Quit => self.received_quit(),
            UCICommand::Show => self.received_show(),
            UCICommand::UCI => self.received_uci(),
        }
    }

    fn uci_position(&mut self, command: PositionCommand) -> Result<(), UCIError> {
        self.board = Board::from_str(&command.fen, self.hasher.clone())?;
        self.board.make_moves(&command.moves)?;
        Ok(())
    }

    // TODO: This should be a separate thread
    fn received_go(&mut self, command: GoCommand) -> Result<(), UCIError> {
        let move_time = match command.move_time {
            Some(time) => time,
            None => 1000,
        };
        let max_nodes = match command.nodes {
            Some(nodes) => nodes,
            None => 0,
        };
        let max_depth = match command.depth {
            Some(depth) => depth,
            None => MAX_DEPTH,
        };
        let infinite = command.infinite;

        let mut moves = Vec::with_capacity(command.search_moves.len());
        for search_move in command.search_moves {
            let mov = Move::parse(&self.board, search_move)?;
            self.board.unmake(mov);

            moves.push(mov);
        }

        let search_info = SearchInfo::new(move_time, max_nodes, max_depth, moves, infinite);

        let mut printer = Printer::new(self.printer.clone());
        let best_move = search(
            self.board.clone(),
            self.cache.clone(),
            search_info,
            &mut printer,
        )?;

        self.println(format!("bestmove {}", best_move))?;

        Ok(())
    }

    fn received_isready(&mut self) -> Result<(), UCIError> {
        self.println("readyok")?;
        Ok(())
    }

    fn received_uci(&mut self) -> Result<(), UCIError> {
        self.printer
            .print(format!("id name {} v{}", NAME, VERSION))?;
        self.printer.print(format!("id author {}", AUTHOR))?;
        self.printer.print(format!("uciok"))?;
        Ok(())
    }

    fn received_uci_new_game(&mut self) -> Result<(), UCIError> {
        // TODO: Do something here
        Ok(())
    }

    fn received_debug(&mut self, command: DebugCommand) -> Result<(), UCIError> {
        self.debug = command.state;
        Ok(())
    }

    fn received_stop(&mut self) -> Result<(), UCIError> {
        // TODO: Destroy every search threads
        Ok(())
    }

    fn received_quit(&mut self) -> Result<(), UCIError> {
        self.println("Exiting the program..")?;
        Ok(())
    }

    pub fn received_show(&mut self) -> Result<(), UCIError> {
        let move_generator = MoveGenerator::new(&self.board);

        self.println(format!("{}", self.board))?;
        self.println(format!("FEN: {}", self.board.to_fen()))?;
        self.println(format!("Hash: 0x{:X}", self.board.hash()))?;

        let is_checkmate = move_generator.is_checkmate(&self.board);
        self.println(format!("Checkmate: {}", is_checkmate))?;
        let is_stalemate = move_generator.is_stalemate(&self.board);
        self.println(format!("Stalemate: {}", is_stalemate))?;

        self.println(format!("Moves {}:", move_generator.len()))?;
        let moves = move_generator
            .map(|mov| mov.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        self.println(format!(" - {}", moves))?;
        self.println("")?;

        let evaluation = evaluate(&self.board, self.board.active());
        self.println(format!("Evaluation for side to move: {}", evaluation))?;

        if let Some(en_passant) = self.board.en_passant() {
            self.println(format!(
                "En passant: Capture {} and move to {}",
                en_passant.to_capture, en_passant.to_move
            ))?;
        }

        Ok(())
    }

    fn received_analyse(&mut self) -> Result<(), UCIError> {
        let mut fen = self.board.to_fen();
        fen = fen.replace(" ", "_");

        let url = format!("{}/{}?color=white", LICHESS_ANALYSIS_BASE, fen);
        if let Err(error) = open::that(url.clone()) {
            self.println("There was an error opening the browser.")?;
            self.send_debug(format!("Error: {}", error))?;
            self.println(format!("Thus here is the link: {}", url))?;
        }

        Ok(())
    }

    fn send_debug(&mut self, message: impl Into<String>) -> Result<(), UCIError> {
        let message = message.into();

        if self.debug {
            self.println(format!("info string {}", message))?;
        }

        Ok(())
    }

    fn println(&mut self, message: impl Into<String>) -> Result<(), UCIError> {
        let message = message.into();
        self.printer.print(message)?;
        Ok(())
    }
}
