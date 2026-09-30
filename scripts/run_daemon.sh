#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

SOCKET_PATH="/tmp/qcos_hilbert_ledger.sock"

echo "=== Building Standalone Hilbert Hotel Ledger Daemon ==="
cd "$PROJECT_ROOT"
cargo build --release

# Clean up stale socket if present
if [ -S "$SOCKET_PATH" ]; then
    echo "Removing existing IPC socket at $SOCKET_PATH..."
    rm -f "$SOCKET_PATH"
fi

echo "=== Starting Standalone Ledger Daemon ==="
./target/release/qcos-hilbert-ledger