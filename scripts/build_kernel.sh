#!/usr/bin/env bash
set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$ROOT_DIR"

echo "=== 1. Building libyashwanth_micro_os.rlib with kernel_bin feature ==="
cargo build --release --lib --features kernel_bin

echo "=== 2. Compiling bare-metal kernel binary with rustc and linker.ld ==="
rustc --edition=2021 src/main.rs \
    --crate-type bin \
    -C opt-level=2 \
    -C panic=abort \
    -C link-arg=-nostdlib \
    -C link-arg=-static \
    -C link-arg=-Tlinker.ld \
    -C link-arg=-no-pie \
    -C link-arg=-z -C link-arg=max-page-size=0x1000 \
    -L dependency=target/release/deps \
    -L dependency=target/release \
    --extern yashwanth_micro_os=target/release/libyashwanth_micro_os.rlib \
    --cfg 'feature="kernel_bin"' \
    -o target/release/yashwanth_micro_os

echo "=== 3. Kernel binary generated successfully ==="
file target/release/yashwanth_micro_os
ls -lh target/release/yashwanth_micro_os
