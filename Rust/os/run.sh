#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BOOT_DIR="$SCRIPT_DIR/../os-boot"
KERNEL="$SCRIPT_DIR/target/x86_64-unknown-none/debug/blog_os"
BIOS_IMAGE="$SCRIPT_DIR/target/x86_64-unknown-none/debug/blog_os-bios.img"

# Build the kernel
cargo build

# Create disk image
cd "$BOOT_DIR" && cargo run -- "$KERNEL"
cd "$SCRIPT_DIR"

# Run in QEMU (serial output to terminal, Ctrl-A then X to exit)
qemu-system-x86_64 -drive format=raw,file="$BIOS_IMAGE" -nographic
