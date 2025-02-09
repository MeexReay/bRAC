#!/usr/bin/env bash

set -e

TARGETS=(
    x86_64-unknown-linux-gnu
    i686-unknown-linux-gnu
    x86_64-pc-windows-gnu
    i686-pc-windows-gnu
    x86_64-apple-darwin
    aarch64-apple-darwin
)

for TARGET in "${TARGETS[@]}"; do
    cargo build --release --target "$TARGET"
done