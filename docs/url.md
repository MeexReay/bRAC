# How does RAC URL work?

RAC URL is used in sRAC and bRAC as the default way of specifying host, running a RAC or WRAC server.

Format of RAC URL:

```
<protocol>://<address>[:<port>]
```

Protocol can be one of these:

|  | **SSL** | **No SSL** |
| :--: | :--: | :--: |
| **WebSocket** | wracs:// | wrac:// |
| **No Websocket** | racs:// | rac:// |