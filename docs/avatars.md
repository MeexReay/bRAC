# avatars

Client just adds this to the end of every message:
```
\x06!!AR!!<avatar url>
```

`\x06` is the control char for ACK \
`<avatar url>` is the url that leads to the raw image for avatar