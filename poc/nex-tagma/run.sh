#!/usr/bin/env bash
set -euo pipefail
# nex-tagma: standalone CI runner
# Usage: ./run.sh

cd "$(dirname "$0")"

echo "=== nex-tagma ==="
cargo build 2>&1 || { echo "nex-tagma: build FAILED"; exit 1; }

echo "--- tests ---"
cargo test --tests 2>&1 || { echo "nex-tagma: tests FAILED"; exit 1; }

echo "--- bench ---"
cargo run -- bench 2>&1 || { echo "nex-tagma: bench FAILED"; exit 1; }

echo "nex-tagma: passed"
