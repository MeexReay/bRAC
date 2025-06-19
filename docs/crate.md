# Using as crate

This article describes how to use the client as rust crate

## Installation

To use exact version:

```toml
[dependencies.bRAC]
git = "https://github.com/MeexReay/bRAC"
tag = "0.1.2+2.0"
default-features = false
```

To use with latest changes:

```toml
[dependencies.bRAC]
git = "https://github.com/MeexReay/bRAC"
default-features = false
```

`default-features = false` here removes GTK4 gui from installation.

## Usage

As the code structure was changed like about gazillion times, 
you need to explore it yourself, if you are using an old version. 
Here is example of usage on commit [80e7b8c](https://github.com/MeexReay/bRAC/commit/80e7b8c50642f9b76be06980305ed03253858d0c)

```rust
use bRAC::proto::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut conn = connect("rac://meex.lol", None)?; // read docs/url.md

    send_message(&mut conn, "<dude> hi RAC-loving kikes!")?;
    register_user(&mut conn, "dude", "password")?;
    send_message_auth(&mut conn, "dude", "password", "my auth message")?;
    send_message_spoof_auth(&mut conn, "<dude> this message totally fucks auth system")?;

    let (mut all_messages, last_size) = read_messages(&mut conn, 10, 0, false)?.unwrap(); // limits with 10 messages

    /* imagine that new messages were written here */

    let (mut new_messages, last_size) = read_messages(&mut conn, 10, last_size, true)?.unwrap(); // chunked reading!

    all_messages.append(&mut new_messages);

    println!("all_messages: {all_messages:?}. last_size: {last_size}");

    Ok(())
}
```

## See more

- [rac-rs - A Rust client library for RAC protocol. (with async support)](https://github.com/kostya-zero/rac-rs)