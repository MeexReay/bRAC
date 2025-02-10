#!/usr/bin/env bash
TARGETS=(
    x86_64-unknown-linux-gnu
    i686-unknown-linux-gnu
    x86_64-pc-windows-gnu
    i686-pc-windows-gnu
)
for TARGET in "${TARGETS[@]}"; do
    cargo build --release --target "$TARGET"
    echo "$TARGET" built
done