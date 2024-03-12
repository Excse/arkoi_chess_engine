#!/bin/bash

PREVIOUS_COMMIT=$(git rev-parse HEAD~1)

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd $SCRIPT_DIR

RELEASE_DIR="../target/release"
ENGINES_DIR="../engines"

# Ensure the engines directory exists
mkdir -p "$ENGINES_DIR"

# Remove any existing engines
rm -rf "$ENGINES_DIR"/*

echo "Building new engine"
cargo build --release
cp "$RELEASE_DIR/ui" "$ENGINES_DIR/new"
git stash

echo "Building old engine with commit $PREVIOUS_COMMIT"
git checkout $PREVIOUS_COMMIT
cargo build --release
cp "$RELEASE_DIR/ui" "$ENGINES_DIR/old"

echo "Resetting to main branch"
git checkout main
git stash pop

echo "Running selfplay"
cutechess-cli \
  -engine conf=ACE_NEW \
  -engine conf=ACE_OLD \
  -each tc=5+0.1 proto=uci \
  -games 2 -rounds 500 -repeat 2 -maxmoves 200 \
  -concurrency 4 \
  -openings file="../books/balsa/Balsa_v110221.pgn" policy=round \
  -pgnout "$ENGINES_DIR/output.pgn" \

ordo -Q -D -a 0 -A "ACE_OLD" -W -n8 -s1000 -U "0,1,2,3,4,5,6,7,8,9,10" -p "$ENGINES_DIR/output.pgn"
