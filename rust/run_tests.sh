#!/usr/bin/env bash
# Runs the full test flow for gdstk-rs on Unix.
#
# Uso:
#   ./run_tests.sh                 # build + test + snapshots
#   RUN_BENCH=1 ./run_tests.sh     # + criterion benchmarks
#   REGENERATE_SNAPSHOTS=1 ./run_tests.sh  # regenerate snapshots

set -euo pipefail

if ! command -v pkg-config >/dev/null 2>&1; then
    echo "pkg-config is required on Unix. Install it together with the zlib and qhull development packages." >&2
    exit 1
fi

echo "=== pkg-config ==="
pkg-config --version

echo "=== Build ==="
cargo build --release --examples

echo ""
echo "=== cargo test --release ==="
cargo test --release

echo ""
echo "=== Benchmarks ==="
if [ "${RUN_BENCH:-0}" = "1" ]; then
    cargo bench
else
    echo "(skipped - set RUN_BENCH=1 to run)"
fi

echo ""
echo "=== ALL DONE ==="
