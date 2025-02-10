@echo off
REM cargo build --release --target x86_64-unknown-linux-gnu
REM echo x86_64-unknown-linux-gnu built
REM cargo build --release --target i686-unknown-linux-gnu
REM echo i686-unknown-linux-gnu built
REM cargo build --release --target x86_64-pc-windows-gnu
REM echo x86_64-pc-windows-gnu built
REM cargo build --release --target i686-pc-windows-gnu
REM echo i686-pc-windows-gnu built
cargo build --release --target x86_64-pc-windows-msvc
echo x86_64-pc-windows-msvc built
cargo build --release --target i686-pc-windows-msvc
echo i686-pc-windows-msvc built