# bRAC
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

## commands

`/clear` - clear chat \
`/spam *args` - spam with text \
`/help` - show help message

## colored usernames

### bRAC

regex - `\uB9AC\u3E70<(.*?)> (.*)` \
color - green \
example - `리㹰<nick> text`

### CRAB

regex - `\u2550\u2550\u2550<(.*?)> (.*)` \
color - light red \
example - `═══<nick> text`

### Mefedroniy

regex - `(.*?): (.*)` \
color - light magenta \
example - `nick: text`

### clRAC

regex - `<(.*?)> (.*)` \
color - cyan \
example - `<nick> text`

## see also

- [RAC protocol (v1.99.2)](https://gitea.bedohswe.eu.org/pixtaded/crab#rac-protocol)
- [CRAB - client & server for RAC](https://gitea.bedohswe.eu.org/pixtaded/crab)
- [AlmatyD - server for RAC](https://gitea.bedohswe.eu.org/bedohswe/almatyd)
- [RAC protocol (v1.0)](https://bedohswe.eu.org/text/rac/protocol.md.html)


