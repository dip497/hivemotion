#!/usr/bin/env bash
# The one local check: formatting, clippy with warnings as errors, and every test.
# Needs ffmpeg and ffprobe on PATH (the tests encode and measure real files).
# The first build downloads a prebuilt FFmpeg for fframes and takes about 10 minutes.
set -euo pipefail
cd "$(dirname "$0")/.."
command -v ffmpeg >/dev/null || { echo "ffmpeg is not on PATH"; exit 1; }
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --release -- -D warnings
cargo test --workspace --release
echo "check: all green"
