use std::{collections::HashMap, fs::File, io::Read, path::Path};

use rand::{seq::SliceRandom, Rng};

use crate::{
    board::{
        color::Color,
        piece::{Piece, Tile},
        Board,
    },
    r#move::Move,
    square::{constants::*, Square},
    zobrist::ZobristHash,
};

use super::{
    error::{InvalidData, InvalidPromotion, NoEntries, PolyglotError},
    hasher::PolyglotHasher,
};

#[derive(Debug)]
pub struct PolyglotMove {
    from_file: u8,
    from_rank: u8,
    to_file: u8,
    to_rank: u8,
    promotion: u8,
}

impl PolyglotMove {
    pub fn parse(data: &[u8]) -> Option<PolyglotMove> {
        if data.len() != 2 {
            return None;
        }

        let move_bytes: [u8; 2] = data[0..2].try_into().ok()?;
        let move_data = u16::from_be_bytes(move_bytes);

        let to_file = (move_data & 0b111) as u8;
        let to_rank = ((move_data >> 3) & 0b111) as u8;

        let from_file = ((move_data >> 6) & 0b111) as u8;
        let from_rank = ((move_data >> 9) & 0b111) as u8;

        let promotion = ((move_data >> 12) & 0b111) as u8;

        Some(Self {
            from_file,
            from_rank,
            to_file,
            to_rank,
            promotion,
        })
    }

    pub fn to_move(&self, board: &Board) -> Result<Move, PolyglotError> {
        let from = Square::new(self.from_rank, self.from_file);
        let mut to = Square::new(self.to_rank, self.to_file);

        // A fix for castling moves
        match (from, to) {
            (E8, H8) => to = G8,
            (E8, A8) => to = C8,
            (E1, H1) => to = G1,
            (E1, A1) => to = C1,
            _ => {}
        }

        let promoted_piece = match self.promotion {
            0 => None,
            1 => Some(Piece::Knight),
            2 => Some(Piece::Bishop),
            3 => Some(Piece::Rook),
            4 => Some(Piece::Queen),
            index => return Err(InvalidPromotion::new(index).into()),
        };

        let mov = if let Some(promoted_piece) = promoted_piece {
            let tile = Tile::new(promoted_piece, Color::Black);
            format!("{}{}{}", from, to, tile.to_fen())
        } else {
            format!("{}{}", from, to)
        };

        let mov = Move::parse(board, mov)?;
        Ok(mov)
    }
}

#[derive(Debug)]
pub struct PolyglotEntry {
    key: u64,
    mov: PolyglotMove,
    weight: u16,
    _learn: u32,
}

impl PolyglotEntry {
    pub fn parse(data: &[u8]) -> Option<PolyglotEntry> {
        if data.len() != 16 {
            return None;
        }

        let key_bytes: [u8; 8] = data[0..8].try_into().ok()?;
        let mov_bytes: [u8; 2] = data[8..10].try_into().ok()?;
        let weight_bytes: [u8; 2] = data[10..12].try_into().ok()?;
        let learn_bytes: [u8; 4] = data[12..16].try_into().ok()?;

        let key = u64::from_be_bytes(key_bytes);
        let weight = u16::from_be_bytes(weight_bytes);
        let _learn = u32::from_be_bytes(learn_bytes);

        let mov_data = u16::from_be_bytes(mov_bytes);
        let mov = PolyglotMove::parse(&mov_data.to_be_bytes())?;

        Some(Self {
            key,
            mov,
            weight,
            _learn,
        })
    }
}

#[derive(Debug)]
pub struct PolyglotBook {
    entries: HashMap<ZobristHash, Vec<PolyglotEntry>>,
}

impl PolyglotBook {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<PolyglotBook, PolyglotError> {
        let path = path.as_ref();
        let mut file = File::open(path)?;

        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        Self::parse(&data)
    }

    pub fn parse(data: &[u8]) -> Result<PolyglotBook, PolyglotError> {
        if data.len() % 16 != 0 {
            return Err(InvalidData.into());
        }

        let parsed_entries = data
            .chunks_exact(16)
            .filter_map(PolyglotEntry::parse)
            .collect::<Vec<PolyglotEntry>>();

        let entries = parsed_entries
            .into_iter()
            .fold(HashMap::new(), |mut acc, entry| {
                let key = ZobristHash::new(entry.key);
                let vec = acc.entry(key).or_insert_with(Vec::new);
                vec.push(entry);
                acc
            });

        Ok(Self { entries })
    }

    pub fn get_entries(&self, board: &Board) -> Result<&Vec<PolyglotEntry>, PolyglotError> {
        let key = PolyglotHasher::hash(&board);

        let entries = self.entries.get(&key);
        if let Some(entries) = entries {
            Ok(entries)
        } else {
            Err(NoEntries.into())
        }
    }

    pub fn get_random_move(&self, board: &Board) -> Result<Move, PolyglotError> {
        let entries = self.get_entries(board)?;
        let mut rand = rand::thread_rng();

        let total_weight = entries.iter().map(|entry| entry.weight as u64).sum::<u64>();
        if total_weight == 0 {
            let entry = entries.choose(&mut rand).unwrap();
            return entry.mov.to_move(board);
        }

        let random_value = rand.gen_range(0..total_weight);

        let mut current_weight = 0;
        for entry in entries {
            current_weight += entry.weight as u64;

            if random_value <= current_weight {
                return entry.mov.to_move(board);
            }
        }

        panic!("No move was selected");
    }
}
