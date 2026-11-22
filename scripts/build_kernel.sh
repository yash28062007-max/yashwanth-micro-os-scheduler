#!/usr/bin/env bash
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$ROOT_DIR"

echo "=== 1. Building bare-metal kernel with cargo for x86_64-unknown-none ==="
cargo build --target x86_64-unknown-none --release --features kernel_bin

mkdir -p target/release
cp target/x86_64-unknown-none/release/yashwanth_micro_os target/release/yashwanth_micro_os

echo "=== 2. Kernel binary generated successfully ==="
file target/x86_64-unknown-none/release/yashwanth_micro_os
ls -lh target/x86_64-unknown-none/release/yashwanth_micro_os
