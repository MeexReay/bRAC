# WRACv2.0 Protocol

Uses websocket for connections, and sends binary data only (works in packet-way manner)

Totally inherits all packets of RACv2, except of reading messages

## Sending messages

Client sends:

- Byte `0x01`
- Message text

## Sending authorized messages

Client sends:

- Byte `0x02`
- Username
- `\n`
- Password
- `\n`
- Message

Server sends:

- nothing if message was sent successfully
- `0x01` if the user does not exists
- `0x02` if the password is incorrect

## Registration users

Client sends:

- Byte `0x03`
- Username
- `\n`
- Password

Server sends:

- nothing if user was registered successfully
- `0x01` if the username is already taken

## Reading messages

### Getting message length

Client sends:

- Byte `0x00`

Server sends:

- Size of all messages in ASCII (data_size)

### Normal reading

This packet is independent from getting message length packet.

Client sends:

- Byte `0x00`
- Byte `0x01`

Server sends:

- All messages

### Chunked reading

This packet is independent from getting message length packet.

Client sends:

- Byte `0x00`
- Byte `0x02`
- Size of messages you have in ASCII (last_size)

Server sends:

- All new messages

*for example: if you want to read last N bytes, last_size = data_size - N*