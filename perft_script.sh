#!/bin/bash

# build the engine in release mode
cargo build --release > /dev/null 2>&1

if [ $# -ge 3 ]; then
  ./target/release/arkoi_chess_engine perft "$1" "$2" "$3" --divide
else 
  ./target/release/arkoi_chess_engine perft "$1" "$2" --divide
fi

