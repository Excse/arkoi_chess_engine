#!/bin/bash

# build the engine in release mode
cargo build --release > /dev/null 2>&1

if [ $# -ge 3 ]; then
  ./target/release/engine perft "$1" "$2" "$3" --divide --hashed --cache-size=4GB
else 
  ./target/release/engine perft "$1" "$2" --divide --hashed --cache-size=4GB
fi

