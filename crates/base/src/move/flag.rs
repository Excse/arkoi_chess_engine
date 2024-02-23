use crate::board::piece::Piece;

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(u8)]
#[rustfmt::skip]
pub enum MoveFlag {
    Quiet                   = 0b0000,
    DoublePawn              = 0b0001,
    KingCastle              = 0b0010,
    QueenCastle             = 0b0011,
    Capture                 = 0b0100,
    EnPassant               = 0b0101,
    KnightPromotion         = 0b1000,
    BishopPromotion         = 0b1001,
    RookPromotion           = 0b1010,
    QueenPromotion          = 0b1011,
    KnightPromotionCapture  = 0b1100,
    BishopPromotionCapture  = 0b1101,
    RookPromotionCapture    = 0b1110,
    QueenPromotionCapture   = 0b1111,
}

impl MoveFlag {
    pub const fn from_flag(flag: u8) -> Self {
        match flag {
            0b0000 => MoveFlag::Quiet,
            0b0001 => MoveFlag::DoublePawn,
            0b0010 => MoveFlag::KingCastle,
            0b0011 => MoveFlag::QueenCastle,
            0b0100 => MoveFlag::Capture,
            0b0101 => MoveFlag::EnPassant,
            0b1000 => MoveFlag::KnightPromotion,
            0b1001 => MoveFlag::BishopPromotion,
            0b1010 => MoveFlag::RookPromotion,
            0b1011 => MoveFlag::QueenPromotion,
            0b1100 => MoveFlag::KnightPromotionCapture,
            0b1101 => MoveFlag::BishopPromotionCapture,
            0b1110 => MoveFlag::RookPromotionCapture,
            0b1111 => MoveFlag::QueenPromotionCapture,
            _ => unreachable!(),
        }
    }

    pub fn get_promotion_piece(&self) -> Piece {
        match self {
            MoveFlag::KnightPromotion | MoveFlag::KnightPromotionCapture => Piece::Knight,
            MoveFlag::BishopPromotion | MoveFlag::BishopPromotionCapture => Piece::Bishop,
            MoveFlag::RookPromotion | MoveFlag::RookPromotionCapture => Piece::Rook,
            MoveFlag::QueenPromotion | MoveFlag::QueenPromotionCapture => Piece::Queen,
            _ => Piece::None,
        }
    }
}
