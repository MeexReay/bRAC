# bRAC
<!--
[<img src="https://github.com/user-attachments/assets/f2be5caa-6246-4a6a-9bee-2b53086f9afb" height="30">]()
[<img src="https://github.com/user-attachments/assets/4d35191d-1dbc-4391-a761-6ae7f76ba7af" height="30">]()
[<img src="https://img.shields.io/badge/Bitcoin-000?style=for-the-badge&logo=bitcoin&logoColor=white">](https://meex.lol/bitcoin)
-->

better RAC client

## features

- gtk4 GUI
- cheat commands (type /help)
- no ip and date visible
- uses TOR proxy server by default (meex.lol:11234)
- plays sound when users receive your messages
- coloring usernames by their clients (CRAB, clRAC, Mefidroniy, etc)
- configurable message format
- RACv1.99.x and RACv2.0 compatible
- RACS compatible (--enable-ssl or in --configure enable SSL)
- chunked reading messages

![screenshot](assets/image.png)

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

3. Build or run with Cargo
```bash
cargo build -r # build release (target/release/bRAC)
cargo run -r # run (builds and runs bRAC itself)
```

TUI version:

```bash
cargo build -r --no-default-features -F tui
cargo run -r --no-default-features -F tui
```

Minimal version:

```bash
cargo build -r --no-default-features
cargo run -r --no-default-features
```

### nix package

If you have Nix package manager installed, you can use:

```bash
nix build github:MeexReay/bRAC # build binary (result/bin/bRAC)
nix run github:MeexReay/bRAC # run (builds and runs bRAC)
```

Minimal version:

```bash
nix build github:MeexReay/bRAC#bRAC-minimal
nix run github:MeexReay/bRAC#bRAC-minimal
```

TUI version:

```bash
nix build github:MeexReay/bRAC#bRAC-tui
nix run github:MeexReay/bRAC#bRAC-tui
```

## default config

get config path - `bRAC --config-path` \
reconfigure client - `bRAC --configure`

```yml
host: meex.lol:11234               # server host
name: null                         # user name (null - ask every time)
message_format: 리㹰<{name}> {text} # message format
update_time: 50                    # update chat interval
max_messages: 200                  # chat messages limit
enable_ip_viewing: true            # enable users' ip viewing
disable_ip_hiding: false           # disable your ip hiding
enable_auth: true                  # enable auth-mode
enable_ssl: false                  # enable ssl connection
enable_chunked: true               # enable chunked reading
```

## command-line options

```
-p, --config-path                      Print config path
-H, --host <HOST>                      Use specified host
-n, --name <NAME>                      Use specified name
-F, --message-format <MESSAGE_FORMAT>  Use specified message format
-r, --read-messages                    Print unformatted messages from chat and exit
-s, --send-message <MESSAGE>           Send unformatted message to chat and exit
-f, --disable-formatting               Disable message formatting and sanitizing
-c, --disable-commands                 Disable slash commands
-i, --disable-ip-hiding                Disable ip hiding
-v, --enable-users-ip-viewing          Enable users IP viewing
-C, --configure                        Configure client
-a, --enable-auth                      Enable authentication
-S, --enable-ssl                       Enable SSL
-u, --enable-chunked                   Enable chunked reading
-h, --help                             Print help
-V, --version                          Print version
```

## cheat commands

commands are any messages that start with a slash `/` \
messages starting with a slash are sent to chat only if the `--disable-commands` option is specified

- `/help` - show help message
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
