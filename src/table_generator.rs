use crate::lookup::generate_lookup_tables;

mod bitboard;
mod board;
mod lookup;
mod move_generator;

fn main() {
    generate_lookup_tables();
}
