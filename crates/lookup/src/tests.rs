#[cfg(test)]
mod tests {
    use crate::generic::BETWEEN;

    #[test]
    fn pawn_pushes() {
        for from in 0..64 {
            for (to, between) in BETWEEN[from].iter().enumerate() {
                println!("[From: {}, To: {}] 0x{:X},", from, to, between);
            }
        }
    }
}
