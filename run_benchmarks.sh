echo "Enter the version of the program: "
read version

temp_file=$(mktemp)
trap "rm -f $temp_file" EXIT

echo "# Benchmarking Version $version" >> BENCHMARK.md

echo "## Startposition:" >> BENCHMARK.md

echo "### Perft(1):" >> BENCHMARK.md
hyperfine --warmup 1 "./perft_script.sh 1 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1'" --export-markdown $temp_file
cat $temp_file >> BENCHMARK.md

echo "### Perft(2):" >> BENCHMARK.md
hyperfine --warmup 1 "./perft_script.sh 2 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1'" --export-markdown $temp_file
cat $temp_file >> BENCHMARK.md

echo "### Perft(3):" >> BENCHMARK.md
hyperfine --warmup 1 "./perft_script.sh 3 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1'" --export-markdown $temp_file
cat $temp_file >> BENCHMARK.md

echo "### Perft(4):" >> BENCHMARK.md
hyperfine --warmup 1 "./perft_script.sh 4 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1'" --export-markdown $temp_file
cat $temp_file >> BENCHMARK.md

echo "### Perft(5):" >> BENCHMARK.md
hyperfine --warmup 1 "./perft_script.sh 5 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1'" --export-markdown $temp_file
cat $temp_file >> BENCHMARK.md
