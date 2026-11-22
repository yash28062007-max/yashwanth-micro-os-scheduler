#!/usr/bin/env bash
# ==============================================================================
# Yashwanth Micro-OS: Safe Environment Setup Script
#
# AUDIT NOTE:
# This script uses strictly official Ubuntu apt packages from canonical mirrors.
# It DOES NOT download or execute any remote script (NO curl | sh).
# ==============================================================================
set -e

echo "=== Step 1: Ensure IPv4 Resolution for Apt ==="
# Command: Writes a single configuration directive to force apt to use IPv4
# Purpose: Avoids IPv6 unreachable route timeouts on WSL virtual adapters
echo 'Acquire::ForceIPv4 "true";' > /etc/apt/apt.conf.d/99force-ipv4

echo "=== Step 2: Update Canonical Package Indexes ==="
# Command: Refreshes Ubuntu package lists
apt-get update -y

echo "=== Step 3: Install Required Build & Emulation Packages ==="
# Packages:
# - rustc, cargo, rust-src: Official Rust compiler and standard library sources
# - build-essential (gcc, ld, make): System linker for x86_64 ELF binaries
# - qemu-system-x86: Hardware emulator for x86_64 bare-metal kernel execution
# - xorriso, mtools: Bootable ISO image creation utilities
# - python3-pip: For Python trace analysis and test automation
DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    rustc \
    cargo \
    rust-src \
    build-essential \
    qemu-system-x86 \
    xorriso \
    mtools \
    python3-pip

echo "=== Step 4: Verification of Installed Toolchain ==="
rustc --version
cargo --version
qemu-system-x86_64 --version
xorriso --version
echo "=== Environment Setup Verified Successfully ==="
