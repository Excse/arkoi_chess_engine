#[cfg(test)]
mod tests {
    use crate::generic::LINE;

    #[test]
    fn pawn_pushes() {
        for (to, between) in LINE[10].iter().enumerate() {
            println!("[From: {}, To: {}] 0x{:X},", 10, to, between);
        }
    }
}
