#!/usr/bin/env bash
# Renders the ProRes master and prints the wall-clock seconds.
set -euo pipefail
mkdir -p build
cargo build --release
start=$(date +%s.%N)
cargo run --release -q -- render -o build/intro.mov
end=$(date +%s.%N)
echo "render seconds: $(echo "$end - $start" | bc)"
