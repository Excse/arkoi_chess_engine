use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

use crossbeam_channel::{select, Receiver, Sender};
use reedline::ExternalPrinter;

use base::{board::Board, r#move::Move, zobrist::ZobristHasher};
use engine::{
    evaluation::evaluate,
    generator::MoveGenerator,
    hashtable::TranspositionTable,
    search::{
        communication::{BestMove, Info, Score, SearchCommand},
        search, SearchInfo, MAX_DEPTH,
    },
};

use super::{
    error::UCIError,
    parser::{DebugCommand, GoCommand, PositionCommand, UCICommand},
};

// Around 32MB
pub const DEFAULT_CACHE_SIZE: usize = 2 << 25;
pub const LICHESS_ANALYSIS_BASE: &str = "https://lichess.org/analysis";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
pub const NAME: &str = env!("CARGO_PKG_NAME");

pub struct UCIController {
    uci_receiver: Receiver<UCICommand>,
    cache: Arc<TranspositionTable>,
    printer: ExternalPrinter<String>,
    hasher: ZobristHasher,
    search_receiver: Receiver<SearchCommand>,
    search_sender: Sender<SearchCommand>,
    search_handle: Option<JoinHandle<()>>,
    board: Board,
    debug: bool,
}

impl UCIController {
    pub fn new(printer: ExternalPrinter<String>, receiver: Receiver<UCICommand>) -> Self {
        let cache = TranspositionTable::size(DEFAULT_CACHE_SIZE);
        let cache = Arc::new(cache);

        let mut rand = rand::thread_rng();
        let hasher = ZobristHasher::new(&mut rand);

        let board = Board::default(hasher.clone());

        let (search_sender, search_receiver) = crossbeam_channel::unbounded();

        Self {
            uci_receiver: receiver,
            cache,
            hasher,
            printer,
            board,
            search_receiver,
            search_sender,
            search_handle: None,
            debug: false,
        }
    }

    pub fn start(&mut self) -> Result<(), UCIError> {
        loop {
            select! {
                recv(self.uci_receiver) -> command => {
                    match command {
                        Ok(command) => self.handle_uci(command)?,
                        Err(error) => panic!("Error in the UCI receiver {}", error)
                    }
                }
                recv(self.search_receiver) -> command => {
                    match command {
                        Ok(command) => self.handle_search(command)?,
                        Err(error) => panic!("Error in the search receiver {}", error)
                    }
                }
            }
        }
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

    fn handle_search(&mut self, command: SearchCommand) -> Result<(), UCIError> {
        match command {
            SearchCommand::BestMove(bestmove) => self.received_bestmove(bestmove),
            SearchCommand::Info(info) => self.received_info(info),
        }
    }

    fn uci_position(&mut self, command: PositionCommand) -> Result<(), UCIError> {
        self.board = Board::from_str(&command.fen, self.hasher.clone())?;
        self.board.make_moves(&command.moves)?;
        Ok(())
    }

    fn received_go(&mut self, command: GoCommand) -> Result<(), UCIError> {
        if let Some(handle) = &self.search_handle {
            if !handle.is_finished() {
                self.println("Thread is already running, please stop it first.")?;
                return Ok(());
            }
        }

        let move_time = match command.move_time {
            Some(time) => time,
            None => 1000,
        };
        let max_nodes = match command.nodes {
            Some(nodes) => nodes,
            None => usize::MAX,
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

        let search_info = SearchInfo::new(
            self.board.clone(),
            self.search_sender.clone(),
            move_time,
            max_nodes,
            max_depth,
            moves,
            infinite,
        );

        let cache = self.cache.clone();
        let handle = thread::spawn(move || search(&cache, search_info).unwrap());
        self.search_handle = Some(handle);

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
        if open::that(url.clone()).is_err() {
            self.println(format!("The link to the board is: {}", url))?;
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

    fn received_bestmove(&mut self, bestmove: BestMove) -> Result<(), UCIError> {
        self.println(format!("bestmove {}", bestmove.mov))?;
        Ok(())
    }

    fn received_info(&mut self, info: Info) -> Result<(), UCIError> {
        let mut result = "info ".to_string();

        if let Some(depth) = info.depth {
            result.push_str(&format!("depth {} ", depth));
        }

        if let Some(time) = info.time {
            result.push_str(&format!("time {} ", time));
        }

        if let Some(nodes) = info.nodes {
            result.push_str(&format!("nodes {} ", nodes));
        }

        if let Some(score) = info.score {
            match score {
                Score::Centipawns(cp) => result.push_str(&format!("score cp {} ", cp)),
                Score::Mate(mate) => result.push_str(&format!("score mate {} ", mate)),
            }
        }

        if let Some(currmove) = info.currmove {
            result.push_str(&format!("currmove {} ", currmove));
        }

        if let Some(currmovenumber) = info.currmovenumber {
            result.push_str(&format!("currmovenumber {} ", currmovenumber));
        }

        if let Some(hashfull) = info.hashfull {
            result.push_str(&format!("hashfull {} ", hashfull));
        }

        if let Some(nps) = info.nps {
            result.push_str(&format!("nps {} ", nps));
        }

        if let Some(pv) = info.pv {
            let pv_string = pv
                .iter()
                .map(|mov| mov.to_string())
                .collect::<Vec<String>>()
                .join(" ");
            result.push_str(&format!("pv {} ", pv_string));
        }

        if let Some(string) = info.string {
            result.push_str(&format!("string {} ", string));
        }

        self.println(result)?;

        Ok(())
    }

    fn println(&mut self, message: impl Into<String>) -> Result<(), UCIError> {
        let message = message.into();
        self.printer.print(message)?;
        Ok(())
    }
}
