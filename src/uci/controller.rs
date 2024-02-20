use std::io::Write;

use crossbeam_channel::Receiver;

use crate::{
    board::{zobrist::ZobristHasher, Board},
    evaluation::evaluate,
    generation::MoveGenerator,
    hashtable::{transposition::TranspositionEntry, HashTable},
    search::search,
};

use super::{error::UCIError, parser::{UCICommand, PositionCommand, GoCommand, DebugCommand}};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
pub const NAME: &str = env!("CARGO_PKG_NAME");

pub struct UCIController<W: Write> {
    receiver: Receiver<UCICommand>,
    cache: HashTable<TranspositionEntry>,
    hasher: ZobristHasher,
    board: Board,
    debug: bool,
    writer: W,
}

impl<W: Write> UCIController<W> {
    pub fn new(writer: W, receiver: Receiver<UCICommand>, cache_size: usize) -> Self {
        let cache = HashTable::size(cache_size);

        let mut rand = rand::thread_rng();
        let hasher = ZobristHasher::new(&mut rand);

        let board = Board::default(hasher.clone());

        Self {
            receiver,
            cache,
            hasher,
            writer,
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
            UCICommand::CacheStats => self.received_cache_stats(),
            UCICommand::Go(command) => self.received_go(command),
            UCICommand::Quit => self.received_stop_search(),
            UCICommand::Stop => self.received_stop_search(),
            UCICommand::IsReady => self.received_isready(),
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
        let best_move = search(&mut self.board, &mut self.cache, &command)?;
        writeln!(self.writer, "bestmove {}", best_move)?;
        Ok(())
    }

    fn received_isready(&mut self) -> Result<(), UCIError> {
        writeln!(self.writer, "readyok")?;
        Ok(())
    }

    fn received_uci(&mut self) -> Result<(), UCIError> {
        writeln!(self.writer, "id name {} v{}", NAME, VERSION)?;
        writeln!(self.writer, "id author {}", AUTHOR)?;
        writeln!(self.writer, "uciok")?;
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

    fn received_stop_search(&mut self) -> Result<(), UCIError> {
        // TODO: Destroy every search threads
        Ok(())
    }

    fn received_cache_stats(&mut self) -> Result<(), UCIError> {
        let misses = self.cache.misses();
        let hits = self.cache.hits();

        let probes = hits + misses;
        let hit_rate = (hits as f64 / probes as f64) * 100.0;

        writeln!(self.writer, "Statistics of the cache usage:")?;
        writeln!(self.writer, " - Hit rate: {:.2}%:", hit_rate)?;
        writeln!(self.writer, "    - Overall probes: {}", probes)?;
        writeln!(self.writer, "    - Hits: {}", hits)?;
        writeln!(self.writer, "    - Misses: {}", misses)?;

        let overwrites = self.cache.overwrites();
        let new = self.cache.new();

        let stores = new + overwrites;
        let overwrite_rate = (overwrites as f64 / stores as f64) * 100.0;

        writeln!(self.writer, " - Overwrite rate: {:.2}%:", overwrite_rate)?;
        writeln!(self.writer, "    - Overall stores: {}", stores)?;
        writeln!(self.writer, "    - Overwrites: {}", overwrites)?;
        writeln!(self.writer, "    - New: {}", new)?;

        Ok(())
    }

    pub fn received_show(&mut self) -> Result<(), UCIError> {
        let moves = MoveGenerator::new(&self.board);

        writeln!(self.writer, "{}", self.board)?;
        writeln!(self.writer, "FEN: {}", self.board.to_fen())?;
        writeln!(self.writer, "Hash: 0x{:X}", self.board.hash())?;

        let is_checkmate = moves.is_checkmate(&self.board);
        writeln!(self.writer, "Checkmate: {}", is_checkmate)?;
        let is_stalemate = moves.is_stalemate(&self.board);
        writeln!(self.writer, "Stalemate: {}", is_stalemate)?;

        writeln!(self.writer, "Moves {}:", moves.len())?;
        write!(self.writer, " - ")?;
        for mov in moves {
            write!(self.writer, "{}, ", mov)?;
        }
        writeln!(self.writer)?;

        let evaluation = evaluate(&self.board, self.board.active());
        writeln!(self.writer, "Evaluation for side to move: {}", evaluation)?;

        if let Some(en_passant) = self.board.en_passant() {
            writeln!(
                self.writer,
                "En passant: Capture {} and move to {}",
                en_passant.to_capture, en_passant.to_move
            )?;
        }

        Ok(())
    }

    fn send_debug(&mut self, message: impl Into<String>) -> Result<(), UCIError> {
        let message = message.into();

        if self.debug {
            writeln!(self.writer, "info string {}", message)?;
        }

        Ok(())
    }
}
