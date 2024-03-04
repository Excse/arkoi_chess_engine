use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
};

use crossbeam_channel::{select, Receiver, Sender};

use base::{
    board::{color::Color, Board},
    polyglot::parser::PolyglotBook,
    r#move::Move,
    zobrist::ZobristHasher,
};
use engine::{
    evaluation::evaluate,
    generator::{AllMoves, MoveGenerator},
    hashtable::TranspositionTable,
    search::{
        communication::{BestMove, Info, Score, SearchCommand},
        error::SearchError,
        search, TimeFrame,
    },
};

use super::{
    error::{OptionValueMissing, UCIError},
    parser::{DebugCommand, GoCommand, PositionCommand, SetOptionCommand, UCICommand},
};

pub const DEFAULLT_BOOK: &[u8; 50032] = include_bytes!("../../books/Perfect2023.bin");
pub const DEFAULT_CACHE_SIZE: usize = 16;
pub const DEFAULT_OWN_BOOK: bool = true;
pub const DEFAULT_THREADS: usize = 1;

pub const LICHESS_ANALYSIS_BASE: &str = "https://lichess.org/analysis";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
pub const NAME: &str = env!("CARGO_PKG_NAME");

pub struct UCIController {
    uci_receiver: Receiver<UCICommand>,
    cache: Arc<TranspositionTable>,
    hasher: ZobristHasher,
    search_receiver: Receiver<SearchCommand>,
    search_sender: Sender<SearchCommand>,
    search_handle: Option<JoinHandle<Result<(), SearchError>>>,
    search_running: Arc<AtomicBool>,
    book: Arc<PolyglotBook>,
    max_threads: usize,
    own_book: bool,
    board: Board,
    debug: bool,
}

impl UCIController {
    pub fn new(uci_receiver: Receiver<UCICommand>) -> Result<Self, UCIError> {
        let (search_sender, search_receiver) = crossbeam_channel::unbounded();

        let cache = TranspositionTable::size(DEFAULT_CACHE_SIZE);
        let book = PolyglotBook::parse(DEFAULLT_BOOK)?;
        let search_running = AtomicBool::new(false);

        let mut rand = rand::thread_rng();
        let hasher = ZobristHasher::random(&mut rand);

        let board = Board::default(hasher.clone());

        Ok(Self {
            uci_receiver,
            cache: Arc::new(cache),
            hasher,
            board,
            search_receiver,
            search_sender,
            search_running: Arc::new(search_running),
            book: Arc::new(book),
            max_threads: DEFAULT_THREADS,
            own_book: DEFAULT_OWN_BOOK,
            search_handle: None,
            debug: false,
        })
    }

    pub fn start(&mut self) -> Result<(), UCIError> {
        loop {
            select! {
                recv(self.uci_receiver) -> command => {
                    match command {
                        Ok(UCICommand::Quit) => {
                            self.handle_uci(UCICommand::Quit)?;
                            return Ok(());
                        },
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
            UCICommand::SetOption(command) => self.received_setoption(command),
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
        if let Some(handle) = self.search_handle.take() {
            handle.join().unwrap()?;
        }

        let mut infinite = command.infinite;
        let time_frame = match command.move_time {
            Some(time) => TimeFrame::new(time),
            None => {
                let time_left = match self.board.active() {
                    Color::White => match command.white_time {
                        Some(time) => time,
                        None => {
                            infinite = true;
                            u128::MAX
                        }
                    },
                    Color::Black => match command.black_time {
                        Some(time) => time,
                        None => {
                            infinite = true;
                            u128::MAX
                        }
                    },
                };
                let increment = match self.board.active() {
                    Color::White => command.white_increment.unwrap_or(0),
                    Color::Black => command.black_increment.unwrap_or(0),
                };

                TimeFrame::estimate(time_left, increment)
            }
        };

        let mut moves = Vec::with_capacity(command.search_moves.len());
        for search_move in command.search_moves {
            let mov = Move::parse(&self.board, search_move)?;

            moves.push(mov);
        }

        let running = self.search_running.clone();
        let sender = self.search_sender.clone();
        let max_threads = self.max_threads;
        let board = self.board.clone();
        let own_book = self.own_book;
        let book = self.book.clone();

        let cache = self.cache.clone();
        let handle = thread::spawn(move || {
            let book = if own_book { Some(book.as_ref()) } else { None };
            search(
                board,
                book,
                cache,
                sender,
                running,
                time_frame,
                command.nodes,
                command.depth,
                moves,
                infinite,
                max_threads,
            )
        });
        self.search_handle = Some(handle);

        Ok(())
    }

    fn received_isready(&mut self) -> Result<(), UCIError> {
        println!("readyok");
        Ok(())
    }

    fn received_uci(&mut self) -> Result<(), UCIError> {
        println!("id name {} v{}", NAME, VERSION);
        println!("id author {}", AUTHOR);

        println!(
            "option name Hash type spin default {} min 1 max 65536",
            DEFAULT_CACHE_SIZE
        );
        println!("option name Clear Hash type button");
        println!(
            "option name Threads type spin default {} min 1 max 128",
            DEFAULT_THREADS
        );
        println!(
            "option name OwnBook type check default {}",
            DEFAULT_OWN_BOOK
        );

        println!("uciok");
        Ok(())
    }

    fn received_setoption(&mut self, command: SetOptionCommand) -> Result<(), UCIError> {
        match command.name.as_str() {
            "Hash" => self.set_hash_size(command.value),
            "Threads" => self.set_threads(command.value),
            "OwnBook" => self.set_own_book(command.value),
            "Clear Hash" => self.clear_hash(),
            _ => todo!(),
        }
    }

    fn set_hash_size(&mut self, value: Option<String>) -> Result<(), UCIError> {
        if let Some(handle) = self.search_handle.take() {
            handle.join().unwrap()?;
        }

        let value = match value {
            Some(value) => value,
            None => return Err(OptionValueMissing.into()),
        };
        let size = value.parse::<usize>()?;

        self.cache = Arc::new(TranspositionTable::size(size));

        Ok(())
    }

    fn set_threads(&mut self, value: Option<String>) -> Result<(), UCIError> {
        if let Some(handle) = self.search_handle.take() {
            handle.join().unwrap()?;
        }

        let value = match value {
            Some(value) => value,
            None => return Err(OptionValueMissing.into()),
        };

        let threads = value.parse::<usize>()?;
        self.max_threads = threads;

        Ok(())
    }

    fn set_own_book(&mut self, value: Option<String>) -> Result<(), UCIError> {
        let value = match value {
            Some(value) => value,
            None => return Err(OptionValueMissing.into()),
        };

        let own_book = value.parse::<bool>()?;
        self.own_book = own_book;

        Ok(())
    }

    fn clear_hash(&mut self) -> Result<(), UCIError> {
        if let Some(handle) = self.search_handle.take() {
            handle.join().unwrap()?;
        }

        self.cache.clear();

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
        self.search_running.store(false, Ordering::Relaxed);

        if let Some(handle) = self.search_handle.take() {
            handle.join().unwrap()?;
        }

        Ok(())
    }

    fn received_quit(&mut self) -> Result<(), UCIError> {
        self.search_running.store(false, Ordering::Relaxed);

        if let Some(handle) = self.search_handle.take() {
            handle.join().unwrap()?;
        }

        Ok(())
    }

    pub fn received_show(&mut self) -> Result<(), UCIError> {
        let move_generator = MoveGenerator::<AllMoves>::new(&self.board);

        println!("{}", self.board);
        println!("FEN: {}", self.board.to_fen());
        println!("Hash: 0x{:X}", self.board.hash());

        let is_checkmate = move_generator.is_checkmate(&self.board);
        println!("Checkmate: {}", is_checkmate);
        let is_stalemate = move_generator.is_stalemate(&self.board);
        println!("Stalemate: {}", is_stalemate);

        println!("Moves {}:", move_generator.len());
        let moves = move_generator
            .map(|mov| mov.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        println!(" - {}", moves);
        println!();

        let evaluation = evaluate(&self.board, self.board.active());
        println!("Evaluation for side to move: {}", evaluation);

        if let Some(en_passant) = self.board.en_passant() {
            println!(
                "En passant: Capture {} and move to {}",
                en_passant.to_capture, en_passant.to_move
            );
        }

        Ok(())
    }

    fn received_analyse(&mut self) -> Result<(), UCIError> {
        let mut fen = self.board.to_fen();
        fen = fen.replace(" ", "_");

        let url = format!("{}/{}?color=white", LICHESS_ANALYSIS_BASE, fen);
        if open::that(url.clone()).is_err() {
            println!("The link to the board is: {}", url);
        }

        Ok(())
    }

    fn send_debug(&mut self, message: impl Into<String>) -> Result<(), UCIError> {
        if !self.debug {
            return Ok(());
        }

        let message = message.into();
        println!("info string {}", message);

        Ok(())
    }

    fn received_bestmove(&mut self, bestmove: BestMove) -> Result<(), UCIError> {
        println!("bestmove {}", bestmove.mov);
        Ok(())
    }

    fn received_info(&mut self, info: Info) -> Result<(), UCIError> {
        print!("info ");

        if let Some(depth) = info.depth {
            print!("depth {} ", depth);

            if let Some(seldepth) = info.seldepth {
                print!("seldepth {} ", seldepth);
            }
        }

        if let Some(time) = info.time {
            print!("time {} ", time);
        }

        if let Some(nodes) = info.nodes {
            print!("nodes {} ", nodes);
        }

        if let Some(score) = info.score {
            match score {
                Score::Centipawns(cp) => print!("score cp {} ", cp),
                Score::Mate(mate) => print!("score mate {} ", mate),
            }
        }

        if let Some(currmove) = info.currmove {
            print!("currmove {} ", currmove);
        }

        if let Some(currmovenumber) = info.currmovenumber {
            print!("currmovenumber {} ", currmovenumber);
        }

        if let Some(hashfull) = info.hashfull {
            print!("hashfull {} ", hashfull);
        }

        if let Some(nps) = info.nps {
            print!("nps {} ", nps);
        }

        if let Some(pv) = info.pv {
            let pv_string = pv
                .iter()
                .map(|mov| mov.to_string())
                .collect::<Vec<String>>()
                .join(" ");
            print!("pv {} ", pv_string);
        }

        if let Some(string) = info.string {
            print!("string {} ", string);
        }

        println!();

        Ok(())
    }
}
