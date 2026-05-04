# CSFS-V1
The code Creator
# CSFS – Cyclic Seeking File System

A circular, desire‑based, domain‑isolated storage engine. Writes are sequential (head), reads can be random within the active ring. Old data is pruned according to its desire level.

## Build

- Install Rust: https://rustup.rs/
- `cargo build --release`
- Binary: `target/release/csfs`

## Usage

```bash
# Format a 100MB file
csfs format test.bin 100

# Write a slice (domain=1, desire=0.8)
csfs write test.bin 1 0.8 "Hello, CSFS"

# Read slice at offset 0 (first slice)
csfs read test.bin 0

# Advance tail, delete slices with desire < 0.5
csfs advance test.bin 0.5
