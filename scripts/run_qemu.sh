#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$ROOT_DIR"

mkdir -p traces results

echo "=== Running Yashwanth Micro-OS in QEMU ==="
QEMU_CMD="qemu-system-x86_64"

if ! command -v $QEMU_CMD &> /dev/null; then
    echo "Error: qemu-system-x86_64 is not installed."
    exit 1
fi

LOG_FILE="traces/qemu_serial.log"
> "$LOG_FILE"

# Determine boot medium: ISO if built, or direct kernel binary
if [ -f "build/yashwanth_micro_os.iso" ]; then
    echo "Booting from ISO: build/yashwanth_micro_os.iso"
    $QEMU_CMD \
        -cpu max \
        -boot d \
        -cdrom build/yashwanth_micro_os.iso \
        -serial stdio \
        -display none \
        -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
        -m 256M \
        -no-reboot \
        | tee "$LOG_FILE" || true
else
    echo "Booting kernel ELF directly: target/x86_64-unknown-none/release/yashwanth_micro_os"
    $QEMU_CMD \
        -kernel target/x86_64-unknown-none/release/yashwanth_micro_os \
        -serial stdio \
        -display none \
        -device isa-debug-exit,iobase=0xf4,iosize=0x04 \
        -m 256M \
        -no-reboot \
        | tee "$LOG_FILE" || true
fi

echo -e "\n[OK] QEMU execution finished. Trace saved to $LOG_FILE"
echo "=== Running Python Trace Analysis ==="
python3 scripts/analyze_traces.py "$LOG_FILE"
