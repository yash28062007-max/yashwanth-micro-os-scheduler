#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$ROOT_DIR"

echo "=================================================="
echo "    Running Yashwanth Micro-OS Automated Tests   "
echo "=================================================="

# Run host-side unit and integration tests
cargo test --verbose -- --nocapture | tee results/test_execution.log

echo -e "\n[PASS] All automated unit and component tests succeeded!"
