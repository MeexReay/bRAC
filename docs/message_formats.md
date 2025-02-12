# message formats

## types

### bRAC

this client

```yml
format: "리㹰<{name}> {text}"
regex:  "\uB9AC\u3E70<(.*?)> (.*)"
color:  "green"
```

### CRAB

[CRAB - client & server for RAC written in java](https://gitea.bedohswe.eu.org/pixtaded/crab)

```yml
format: "═══<{name}> {text}"
regex:  "\u2550\u2550\u2550<(.*?)> (.*)"
color:  "light red"
```

### Mefedroniy

[Mefidroniy - client for RAC written in rust](https://github.com/OctoBanon-Main/mefedroniy-client)

```yml
format: "°ʘ<{name}> {text}"
regex:  "\u00B0\u0298<(.*?)> (.*)"
color:  "light magenta"
```

### clRAC

official client

```yml
format: "<{name}> {text}"
regex:  "<(.*?)> (.*)"
color:  "cyan"
```

## developer notes

in auth-mode, there is must to be `> ` after name (`{name}> {text}`)