#!/usr/bin/env bash
cargo build --release --target x86_64-unknown-linux-gnu
echo x86_64-unknown-linux-gnu built
cargo build --release --target i686-unknown-linux-gnu
echo i686-unknown-linux-gnu built
cargo build --release --target x86_64-pc-windows-gnu
echo x86_64-pc-windows-gnu built
# cargo build --release --target i686-pc-windows-gnu
# echo i686-pc-windows-gnu built
# cargo build --release --target x86_64-pc-windows-msvc
# echo x86_64-pc-windows-msvc built
# cargo build --release --target i686-pc-windows-msvc
# echo i686-pc-windows-msvc built