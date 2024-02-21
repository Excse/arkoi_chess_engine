#[cfg(test)]
mod tests {
    use crate::magic::BISHOP_ATTACKS;

    #[test]
    fn pawn_pushes() {
        for (to, between) in BISHOP_ATTACKS[0][0..20].iter().enumerate() {
            println!("[From: {}, To: {}] 0x{:X},", 0, to, between);
        }
    }
}
