pub(crate) const fn rank_file(square: usize) -> (usize, usize) {
    (square / 8, square % 8)
}

pub(crate) const fn index(rank: usize, file: usize) -> usize {
    rank * 8 + file
}

pub(crate) const fn bits(rank: usize, file: usize) -> u64 {
    1 << index(rank, file)
}

pub(crate) const fn random_64_few_bits(seed: u32) -> (u64, u32) {
    let (first, seed) = random_64(seed);
    let (second, seed) = random_64(seed);
    let (third, seed) = random_64(seed);
    let random = first & second & third;
    (random, seed)
}

pub(crate) const fn random_64(mut seed: u32) -> (u64, u32) {
    seed = xor_shift(seed);
    let first = (seed as u64) & 0xFFFF;
    seed = xor_shift(seed);
    let second = (seed as u64) & 0xFFFF;
    seed = xor_shift(seed);
    let third = (seed as u64) & 0xFFFF;
    seed = xor_shift(seed);
    let fourth = (seed as u64) & 0xFFFF;

    let random = first | (second << 16) | (third << 32) | (fourth << 48);
    (random, seed)
}

pub(crate) const fn xor_shift(mut input: u32) -> u32 {
    input ^= input >> 13;
    input ^= input << 17;
    input ^= input >> 5;
    input
}
