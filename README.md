# ![logo](https://raw.githubusercontent.com/MeexReay/bRAC/refs/heads/main/logo.gif)
<!--
[<img src="https://github.com/user-attachments/assets/f2be5caa-6246-4a6a-9bee-2b53086f9afb" height="30">]()
[<img src="https://github.com/user-attachments/assets/4d35191d-1dbc-4391-a761-6ae7f76ba7af" height="30">]()
[<img src="https://img.shields.io/badge/Bitcoin-000?style=for-the-badge&logo=bitcoin&logoColor=white">](https://meex.lol/bitcoin)
-->

better RAC client

## features

- gtk4 GUI
- fancy TUI version
- RACv1.99.x and RACv2.0 compatible
- chat commands (type /help)
- no ip and date visible for anyone
- uses TOR proxy server by default (meex.lol:11234)
- coloring usernames by their clients (CRAB, clRAC, Mefidroniy, etc)
- many command-line options (--help)
- rich configuration (--config-path to get file path and --configure to edit)
- RACS compatible (--enable-ssl or in --configure enable SSL)
- chunked reading messages

![screenshot](image.png)

## how to run

### download binary

go to [releases](https://github.com/MeexReay/bRAC/releases/latest) and download file you need. its simple.

### build from source

1. Make sure [Rust](https://www.rust-lang.org/tools/install) is installed

2. Clone repository
```bash
git clone https://github.com/MeexReay/bRAC.git
cd bRAC
```

3. Run with Cargo
```bash
cargo run -r                              # run GUI version
cargo run -r --no-default-features -F tui # run TUI version
cargo run -r --no-default-features        # run minimal version

# change "cargo run" to "cargo build" to just build (target/release/bRAC)
```

### nix package

If you have Nix package manager installed, you can use:

```bash
nix run github:MeexReay/bRAC                # run GUI version
nix run github:MeexReay/bRAC#bRAC-tui       # run TUI version
nix run github:MeexReay/bRAC#bRAC-minimal   # run minimal version

# change "nix run" to "nix build" to just build (result/bin/bRAC)
```

## chat commands

commands are any messages that start with a slash `/` \
messages starting with a slash are sent to chat only if the `--disable-commands` option is specified

- `/help` - show help message
- `/register password` - try to register account
- `/login password` - login to account
- `/clear` - clear chat
- `/spam *args` - spam with text
- `/ping` - get server ping (send + read)

## docs

- [Message formats](https://github.com/MeexReay/bRAC/blob/main/docs/message_formats.md)
- [Authenticated mode](https://github.com/MeexReay/bRAC/blob/main/docs/auth_mode.md)

## see also

- [RAC-Hub - all about RAC protocol](https://forbirdden.github.io/RAC-Hub/)
- [RAC protocol (v2.0)](https://gitea.bedohswe.eu.org/pixtaded/crab#rac-protocol)
- [CRAB - client & server for RAC](https://gitea.bedohswe.eu.org/pixtaded/crab)
- [Mefidroniy - client for RAC](https://github.com/OctoBanon-Main/mefedroniy-client)
- [AlmatyD - server for RACv1.0](https://gitea.bedohswe.eu.org/bedohswe/almatyd)
- [RAC protocol (v1.0)](https://bedohswe.eu.org/text/rac/protocol.md.html)
