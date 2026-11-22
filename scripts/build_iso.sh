#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$ROOT_DIR"

echo "=== 1. Building Yashwanth Micro-OS Kernel Binary ==="
bash "$SCRIPT_DIR/build_kernel.sh"

echo "=== 2. Preparing ISO Directory Structure ==="
rm -rf build/iso_root
mkdir -p build/iso_root/boot/limine
mkdir -p build/iso_root/EFI/BOOT
mkdir -p build/limine

cp target/release/yashwanth_micro_os build/iso_root/boot/
cp limine.conf build/iso_root/boot/limine/limine.conf
cp limine.conf build/iso_root/boot/limine/limine.cfg
cp limine.conf build/iso_root/limine.cfg

# Prepare Limine binaries
if [ ! -f "build/limine/limine" ]; then
    echo "Cloning Limine binaries..."
    if [ ! -d "build/limine_repo" ]; then
        git clone https://github.com/limine-bootloader/limine.git --branch=v7.x-binary --depth=1 build/limine_repo
    fi
    gcc -O2 build/limine_repo/limine.c -o build/limine/limine
fi

cp build/limine_repo/limine-bios.sys build/iso_root/boot/limine/
cp build/limine_repo/limine-bios-cd.bin build/iso_root/boot/limine/
cp build/limine_repo/limine-uefi-cd.bin build/iso_root/boot/limine/
cp build/limine_repo/BOOTX64.EFI build/iso_root/EFI/BOOT/

echo "=== 3. Creating Bootable Hybrid ISO with xorriso ==="
xorriso -as mkisofs \
    -b boot/limine/limine-bios-cd.bin \
    -no-emul-boot \
    -boot-load-size 4 \
    -boot-info-table \
    --efi-boot boot/limine/limine-uefi-cd.bin \
    -efi-boot-part \
    --efi-boot-image \
    --protective-msdos-label \
    build/iso_root \
    -o build/yashwanth_micro_os.iso

echo "=== 4. Stamping BIOS Bootloader onto ISO ==="
build/limine/limine bios-install build/yashwanth_micro_os.iso

echo "=== ISO Build Complete: build/yashwanth_micro_os.iso ==="
ls -lh build/yashwanth_micro_os.iso
