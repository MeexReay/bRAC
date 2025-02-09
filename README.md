# bRAC
better RAC client

## features

- no ip and date visible
- plays sound on receiving messages
- using tor proxy by default
- coloring usernames by their clients (CRAB, clRAC, Mefidroniy, etc)
- RACv1.99.x compatible

![image](https://github.com/user-attachments/assets/a2858662-50f1-4554-949c-f55addf48fcc)

## how to use

(you have to install [rust](https://rust-lang.org) at first)

```bash
cargo build # build
cargo run   # run
```

## extensions

### colored usernames

#### bRAC

regex - `\[(.*?)\] \uB9AC\u3E70<(.*?)> (.*)` \
color - green \
example - `[date] 리㹰<nick> text`

#### CRAB

regex - `\[(.*?)\] \u2550\u2550\u2550<(.*?)> (.*)` \
color - light red \
example - `[date] ═══<nick> text`

#### Mefedroniy

regex - `\[(.*?)\] (.*?): (.*)` \
color - light magenta \
example - `[date] nick: text`

#### clRAC

regex - `\[(.*?)\] <(.*?)> (.*)` \
color - cyan \
example - `[date] <nick> text`

## see also

- [CRAB - client & server for RAC](https://gitea.bedohswe.eu.org/pixtaded/crab)
- [AlmatyD - server for RAC](https://gitea.bedohswe.eu.org/bedohswe/almatyd)
- [RAC protocol (v1.0)](https://bedohswe.eu.org/text/rac/protocol.md.html)
- [RAC protocol (v1.99.1)](https://gitea.bedohswe.eu.org/)


