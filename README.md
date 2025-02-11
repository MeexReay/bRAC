# bRAC
[<img src="https://github.com/user-attachments/assets/f2be5caa-6246-4a6a-9bee-2b53086f9afb" height="30">]()
[<img src="https://github.com/user-attachments/assets/4d35191d-1dbc-4391-a761-6ae7f76ba7af" height="30">]()
[<img src="https://img.shields.io/badge/Bitcoin-000?style=for-the-badge&logo=bitcoin&logoColor=white">](https://meex.lol/bitcoin)

better RAC client

## features

- cheat commands (type /help)
- no ip and date visible
- uses TOR proxy server by default
- plays sound when users receive your messages
- coloring usernames by their clients (CRAB, clRAC, Mefidroniy, etc)
- configurable message format 
- RACv1.99.x compatible

![image](https://github.com/user-attachments/assets/a2858662-50f1-4554-949c-f55addf48fcc)

## how to run

### download binary

go to [releases](https://github.com/MeexReay/bRAC/releases/latest) and download file you need. its simple.

### build from source

(you have to install [rust](https://www.rust-lang.org/tools/install) at first)

```bash
git clone https://github.com/MeexReay/bRAC.git
cd bRAC
cargo build --release # build release (target/release/bRAC)
cargo run   # run (builds and runs bRAC itself)
```

## config

```yml
host: meex.lol:11234               # server host
name: null                         # user name
message_format: 리㹰<{name}> {text} # message format
update_time: 50                    # update chat interval
max_messages: 100                  # chat messages limit
```

## command args

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
-h, --help                             Print help
-V, --version                          Print version
```

## commands

`/help` - show help message \
`/clear` - clear chat \
`/spam *args` - spam with text \
`/ping` - get server ping (send + read)

## see also

- [RAC protocol (v1.99.2)](https://gitea.bedohswe.eu.org/pixtaded/crab#rac-protocol)
- [CRAB - client & server for RAC](https://gitea.bedohswe.eu.org/pixtaded/crab)
- [Colored usernames](https://github.com/MeexReay/bRAC/blob/main/docs/colored_usernames.md)
- [AlmatyD - server for RAC](https://gitea.bedohswe.eu.org/bedohswe/almatyd)
- [RAC protocol (v1.0)](https://bedohswe.eu.org/text/rac/protocol.md.html)
