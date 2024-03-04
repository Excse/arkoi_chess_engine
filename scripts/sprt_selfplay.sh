#!/bin/bash

CURRENT_COMMIT=$(git rev-parse HEAD)
PREVIOUS_COMMIT=$(git rev-parse HEAD~1)

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
RELEASE_DIR="$SCRIPT_DIR/../target/release"
ENGINES_DIR="$SCRIPT_DIR/../engines"

# Ensure the engines directory exists
mkdir -p "$ENGINES_DIR"

# Remove any existing engines
rm -rf "$ENGINES_DIR"/*

echo "Building new engine with commit $CURRENT_COMMIT"
git checkout $CURRENT_COMMIT
cargo build --release
cp "$RELEASE_DIR/ui" "$ENGINES_DIR/new"

echo "Building old engine with commit $PREVIOUS_COMMIT"
git checkout $PREVIOUS_COMMIT
cargo build --release
cp "$RELEASE_DIR/ui" "$ENGINES_DIR/old"

echo "Resetting to main branch"
git checkout main

echo "Running selfplay"
cutechess-cli \
  -engine cmd="$ENGINES_DIR/old" name="ACE Old" \
  -engine cmd="$ENGINES_DIR/new" name="ACE New" \
  -each tc=inf/10+0.1 proto=uci \
  -games 2 -rounds 500 -repeat 2 -maxmoves 200 \
  -concurrency 8 \
  -ratinginterval 10 \
  -pgnout "output.pgn"
