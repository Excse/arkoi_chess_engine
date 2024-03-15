#!/bin/bash

# Function to check if a commit hash exists
check_commit() {
    local commit_hash="$1"
    if git rev-parse --quiet --verify "$commit_hash" >/dev/null; then
        echo "Commit hash $commit_hash exists."
    else
        echo "Error: Commit hash $commit_hash does not exist."
        exit 1
    fi
}

# Ask the user if they want to use the previous commit
read -p "Do you want to use the previous commit? (y/n): " use_previous

if [[ $use_previous == "y" || $use_previous == "Y" ]]; then
    prev_commit=$(git rev-parse HEAD^)
    echo "Using previous commit: $prev_commit"
    check_commit "$prev_commit"
    chosen_commit="$prev_commit"
elif [[ $use_previous == "n" || $use_previous == "N" ]]; then
    # Ask the user for input
    read -p "Enter the commit hash to check: " input_commit
    check_commit "$input_commit"
    chosen_commit="$input_commit"
else
    echo "Invalid input. Please enter 'y' or 'n'."
    exit 1
fi

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

echo "Building old engine with commit $chosen_commit"
git checkout $chosen_commit
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
  -openings file="../books/UHO/UHO_4060_v3.epd" policy=round \
  -pgnout "$ENGINES_DIR/output.pgn" \

ordo -Q -D -a 0 -A "ACE_OLD" -W -n8 -s1000 -U "0,1,2,3,4,5,6,7,8,9,10" -p "$ENGINES_DIR/output.pgn"
