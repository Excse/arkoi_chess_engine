#!/bin/bash

# build the engine in release mode
cargo build -p engine --release > /dev/null 2>&1

if [ $# -ge 3 ]; then
  ./target/release/engine "$1" "$2" "$3"
else 
  ./target/release/engine "$1" "$2"
fi

