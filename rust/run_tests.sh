#!/usr/bin/env bash
# Corre el flujo completo de testing para gdstk-rs.
#
# Uso:
#   ./run_tests.sh            # build + test + snapshots
#   RUN_BENCH=1 ./run_tests.sh  # + criterion benchmarks (tarda ~30s)
#   REGENERATE_SNAPSHOTS=1 ./run_tests.sh  # regenera snapshots

set -euo pipefail

export VCPKG_ROOT="${VCPKG_ROOT:-C:/vcpkg}"
VCPKG_BIN="${VCPKG_ROOT}/installed/x64-windows/bin"

echo "=== Build ==="
cargo build --release --examples

# Copiar DLLs necesarias en Windows (idempotente, tolera fallo si ya están).
if [ -d "$VCPKG_BIN" ]; then
    cp -f "$VCPKG_BIN/zlib1.dll" target/release/examples/ 2>/dev/null || true
    cp -f "$VCPKG_BIN/qhull_r.dll" target/release/examples/ 2>/dev/null || true
fi

echo ""
echo "=== cargo test --release ==="
cargo test --release

echo ""
echo "=== Benchmarks ==="
if [ "${RUN_BENCH:-0}" = "1" ]; then
    cargo bench
else
    echo "(skipped — set RUN_BENCH=1 para correr)"
fi

echo ""
echo "=== TODO PASO OK ==="
