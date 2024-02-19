use super::{square::Square, Bitboard};

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
pub const RANKS: [Bitboard; 8] = [
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
pub const FILES: [Bitboard; 8] = [
    FILE_A, FILE_B, FILE_C, FILE_D, FILE_E, FILE_F, FILE_G, FILE_H,
];

#[allow(dead_code)]
pub const A1: Square = Square::from_index(0);
#[allow(dead_code)]
pub const B1: Square = Square::from_index(1);
#[allow(dead_code)]
pub const C1: Square = Square::from_index(2);
#[allow(dead_code)]
pub const D1: Square = Square::from_index(3);
#[allow(dead_code)]
pub const E1: Square = Square::from_index(4);
#[allow(dead_code)]
pub const F1: Square = Square::from_index(5);
#[allow(dead_code)]
pub const G1: Square = Square::from_index(6);
#[allow(dead_code)]
pub const H1: Square = Square::from_index(7);

#[allow(dead_code)]
pub const A2: Square = Square::from_index(8);
#[allow(dead_code)]
pub const B2: Square = Square::from_index(9);
#[allow(dead_code)]
pub const C2: Square = Square::from_index(10);
#[allow(dead_code)]
pub const D2: Square = Square::from_index(11);
#[allow(dead_code)]
pub const E2: Square = Square::from_index(12);
#[allow(dead_code)]
pub const F2: Square = Square::from_index(13);
#[allow(dead_code)]
pub const G2: Square = Square::from_index(14);
#[allow(dead_code)]
pub const H2: Square = Square::from_index(15);

#[allow(dead_code)]
pub const A3: Square = Square::from_index(16);
#[allow(dead_code)]
pub const B3: Square = Square::from_index(17);
#[allow(dead_code)]
pub const C3: Square = Square::from_index(18);
#[allow(dead_code)]
pub const D3: Square = Square::from_index(19);
#[allow(dead_code)]
pub const E3: Square = Square::from_index(20);
#[allow(dead_code)]
pub const F3: Square = Square::from_index(21);
#[allow(dead_code)]
pub const G3: Square = Square::from_index(22);
#[allow(dead_code)]
pub const H3: Square = Square::from_index(23);

#[allow(dead_code)]
pub const A4: Square = Square::from_index(24);
#[allow(dead_code)]
pub const B4: Square = Square::from_index(25);
#[allow(dead_code)]
pub const C4: Square = Square::from_index(26);
#[allow(dead_code)]
pub const D4: Square = Square::from_index(27);
#[allow(dead_code)]
pub const E4: Square = Square::from_index(28);
#[allow(dead_code)]
pub const F4: Square = Square::from_index(29);
#[allow(dead_code)]
pub const G4: Square = Square::from_index(30);
#[allow(dead_code)]
pub const H4: Square = Square::from_index(31);

#[allow(dead_code)]
pub const A5: Square = Square::from_index(32);
#[allow(dead_code)]
pub const B5: Square = Square::from_index(33);
#[allow(dead_code)]
pub const C5: Square = Square::from_index(34);
#[allow(dead_code)]
pub const D5: Square = Square::from_index(35);
#[allow(dead_code)]
pub const E5: Square = Square::from_index(36);
#[allow(dead_code)]
pub const F5: Square = Square::from_index(37);
#[allow(dead_code)]
pub const G5: Square = Square::from_index(38);
#[allow(dead_code)]
pub const H5: Square = Square::from_index(39);

#[allow(dead_code)]
pub const A6: Square = Square::from_index(40);
#[allow(dead_code)]
pub const B6: Square = Square::from_index(41);
#[allow(dead_code)]
pub const C6: Square = Square::from_index(42);
#[allow(dead_code)]
pub const D6: Square = Square::from_index(43);
#[allow(dead_code)]
pub const E6: Square = Square::from_index(44);
#[allow(dead_code)]
pub const F6: Square = Square::from_index(45);
#[allow(dead_code)]
pub const G6: Square = Square::from_index(46);
#[allow(dead_code)]
pub const H6: Square = Square::from_index(47);

#[allow(dead_code)]
pub const A7: Square = Square::from_index(48);
#[allow(dead_code)]
pub const B7: Square = Square::from_index(49);
#[allow(dead_code)]
pub const C7: Square = Square::from_index(50);
#[allow(dead_code)]
pub const D7: Square = Square::from_index(51);
#[allow(dead_code)]
pub const E7: Square = Square::from_index(52);
#[allow(dead_code)]
pub const F7: Square = Square::from_index(53);
#[allow(dead_code)]
pub const G7: Square = Square::from_index(54);
#[allow(dead_code)]
pub const H7: Square = Square::from_index(55);

#[allow(dead_code)]
pub const A8: Square = Square::from_index(56);
#[allow(dead_code)]
pub const B8: Square = Square::from_index(57);
#[allow(dead_code)]
pub const C8: Square = Square::from_index(58);
#[allow(dead_code)]
pub const D8: Square = Square::from_index(59);
#[allow(dead_code)]
pub const E8: Square = Square::from_index(60);
#[allow(dead_code)]
pub const F8: Square = Square::from_index(61);
#[allow(dead_code)]
pub const G8: Square = Square::from_index(62);
#[allow(dead_code)]
pub const H8: Square = Square::from_index(63);
