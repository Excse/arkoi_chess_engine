use super::Bitboard;

#[allow(dead_code)]
pub const RANK_1: Bitboard = Bitboard::from_bits(0xFF);
#[allow(dead_code)]
pub const RANK_2: Bitboard = Bitboard::from_bits(0xFF00);
#[allow(dead_code)]
pub const RANK_3: Bitboard = Bitboard::from_bits(0xFF0000);
#[allow(dead_code)]
pub const RANK_4: Bitboard = Bitboard::from_bits(0xFF000000);
#[allow(dead_code)]
pub const RANK_5: Bitboard = Bitboard::from_bits(0xFF00000000);
#[allow(dead_code)]
pub const RANK_6: Bitboard = Bitboard::from_bits(0xFF0000000000);
#[allow(dead_code)]
pub const RANK_7: Bitboard = Bitboard::from_bits(0xFF000000000000);
#[allow(dead_code)]
pub const RANK_8: Bitboard = Bitboard::from_bits(0xFF00000000000000);

#[allow(dead_code)]
pub const RANKS_AMOUNT: usize = 8;

#[allow(dead_code)]
pub const RANKS: [Bitboard; RANKS_AMOUNT] = [
    RANK_1, RANK_2, RANK_3, RANK_4, RANK_5, RANK_6, RANK_7, RANK_8,
];

#[allow(dead_code)]
pub const FILE_A: Bitboard = Bitboard::from_bits(0x101010101010101);
#[allow(dead_code)]
pub const FILE_B: Bitboard = Bitboard::from_bits(0x202020202020202);
#[allow(dead_code)]
pub const FILE_C: Bitboard = Bitboard::from_bits(0x404040404040404);
#[allow(dead_code)]
pub const FILE_D: Bitboard = Bitboard::from_bits(0x808080808080808);
#[allow(dead_code)]
pub const FILE_E: Bitboard = Bitboard::from_bits(0x1010101010101010);
#[allow(dead_code)]
pub const FILE_F: Bitboard = Bitboard::from_bits(0x2020202020202020);
#[allow(dead_code)]
pub const FILE_G: Bitboard = Bitboard::from_bits(0x4040404040404040);
#[allow(dead_code)]
pub const FILE_H: Bitboard = Bitboard::from_bits(0x8080808080808080);

#[allow(dead_code)]
pub const FILES_AMOUNT: usize = 8;

#[allow(dead_code)]
pub const FILES: [Bitboard; FILES_AMOUNT] = [
    FILE_A, FILE_B, FILE_C, FILE_D, FILE_E, FILE_F, FILE_G, FILE_H,
];
