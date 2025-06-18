# ![logo](misc/logo.gif)
<!--
[<img src="https://github.com/user-attachments/assets/f2be5caa-6246-4a6a-9bee-2b53086f9afb" height="30">]()
[<img src="https://github.com/user-attachments/assets/4d35191d-1dbc-4391-a761-6ae7f76ba7af" height="30">]()
[<img src="https://img.shields.io/badge/Bitcoin-000?style=for-the-badge&logo=bitcoin&logoColor=white">](https://meex.lol/bitcoin)
-->

better RAC client

## features

- gtk4 modern GUI
- RACv1.99.x and RACv2.0 compatible
- WRAC compatible ([docs](docs/wrac.md))
- chat commands (type /help)
- no ip and date visible for anyone (almost)
- coloring usernames by their clients (CRAB, clRAC, Mefidroniy, etc.)
- many command-line options (see --help)
- rich configuration (--config-path to get file path)
- RACS/WRACS compatible (ex: wracs://meex.lol)
- reading messages chunked (less traffic usage)

![screenshot](misc/image.png)

## how to run

### install (recommended)

**for linux:**

1. Install requirements: `gtk4-dev gtk4 make rust git openssl-dev openssl`

2. Clone repository and run `make install`

3. Now you are able to start bRAC with desktop icon

**for windows:**

NO SOLUTION

Read [compiling docs](docs/compiling.md) to build it manually.

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
cargo build -r # build release (target/release/bRAC) 
cargo run -r # build and run
```

Read more about that on the [compiling docs](docs/compiling.md).

### nix package

If you have Nix package manager installed, you can use:

```bash
nix build github:MeexReay/bRAC # build release (result/bin/bRAC)
nix run github:MeexReay/bRAC # build and run
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

- [Compiling](docs/compiling.md)
- [User agents](docs/user_agents.md)
- [Authenticated mode](docs/auth_mode.md)
- [WRAC protocol (v2.0)](docs/wrac.md)
- [About RAC URL](docs/url.md)
- [FAQ](docs/faq.md)

## see also

- [RAC-Hub - all about RAC protocol](https://the-stratosphere-solutions.github.io/RAC-Hub/)
- [RAC protocol (v2.0)](https://gitea.bedohswe.eu.org/pixtaded/crab#rac-protocol)
- [CRAB - client & server for RAC](https://gitea.bedohswe.eu.org/pixtaded/crab)
- [Mefidroniy - client for RAC](https://github.com/OctoBanon-Main/mefedroniy-client)
- [cRACk - client for RAC-loving kikes](https://github.com/pansangg/cRACk)
- [AlmatyD - server for RACv1.0](https://gitea.bedohswe.eu.org/bedohswe/almatyd)
- [RAC protocol (v1.0)](https://bedohswe.eu.org/text/rac/protocol.md.html)
