use std::{
    error::Error,
    fmt::Debug,
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
    time::Duration,
};

use native_tls::{TlsConnector, TlsStream};
use socks::Socks5Stream;
use tungstenite::{client::client_with_config, protocol::WebSocketConfig, WebSocket};

pub mod rac;
pub mod wrac;

pub trait Stream: Read + Write + Unpin + Send + Sync + Debug {
    fn set_read_timeout(&self, timeout: Duration);
    fn set_write_timeout(&self, timeout: Duration);
}

impl Stream for TcpStream {
    fn set_read_timeout(&self, timeout: Duration) {
        let _ = TcpStream::set_read_timeout(&self, Some(timeout));
    }
    fn set_write_timeout(&self, timeout: Duration) {
        let _ = TcpStream::set_write_timeout(&self, Some(timeout));
    }
}

impl Stream for Socks5Stream {
    fn set_read_timeout(&self, timeout: Duration) {
        let _ = TcpStream::set_read_timeout(self.get_ref(), Some(timeout));
    }
    fn set_write_timeout(&self, timeout: Duration) {
        let _ = TcpStream::set_write_timeout(self.get_ref(), Some(timeout));
    }
}

impl<T: Stream> Stream for TlsStream<T> {
    fn set_read_timeout(&self, timeout: Duration) {
        self.get_ref().set_read_timeout(timeout);
    }
    fn set_write_timeout(&self, timeout: Duration) {
        self.get_ref().set_write_timeout(timeout);
    }
}

impl Stream for TlsStream<Box<dyn Stream>> {
    fn set_read_timeout(&self, timeout: Duration) {
        self.get_ref().set_read_timeout(timeout);
    }
    fn set_write_timeout(&self, timeout: Duration) {
        self.get_ref().set_write_timeout(timeout);
    }
}

pub enum RacStream {
    WRAC(WebSocket<Box<dyn Stream>>),
    RAC(Box<dyn Stream>),
}

/// `socks5://user:pass@127.0.0.1:12345/path -> ("127.0.0.1:12345", ("user", "pass"))` \
/// `socks5://127.0.0.1:12345 -> ("127.0.0.1:12345", None)` \
/// `https://127.0.0.1:12345 -> ("127.0.0.1:12345", None)` \
/// `127.0.0.1:12345 -> ("127.0.0.1:12345", None)` \
/// `user:pass@127.0.0.1:12345 -> ("127.0.0.1:12345", ("user", "pass"))`
pub fn parse_socks5_url(url: &str) -> Option<(String, Option<(String, String)>)> {
    let (_, url) = url.split_once("://").unwrap_or(("", url));
    let (url, _) = url.split_once("/").unwrap_or((url, ""));
    if let Some((auth, url)) = url.split_once("@") {
        let (user, pass) = auth.split_once(":")?;
        Some((url.to_string(), Some((user.to_string(), pass.to_string()))))
    } else {
        Some((url.to_string(), None))
    }
}

/// url -> (host, ssl, wrac) \
/// `127.0.0.1` -> `("127.0.0.1:42666", false, false)` \
/// `127.0.0.1:12345` -> `("127.0.0.1:12345", false, false)` \
/// `rac://127.0.0.1/` -> `("127.0.0.1:42666", false, false)` \
/// `racs://127.0.0.1/` -> `("127.0.0.1:42667", true, false)` \
/// `wrac://127.0.0.1/` -> `("127.0.0.1:52666", false, true)` \
/// `wracs://127.0.0.1/` -> `(127.0.0.1:52667, true, true)` \
pub fn parse_rac_url(url: &str) -> Option<(String, bool, bool)> {
    let (scheme, url) = url.split_once("://").unwrap_or(("rac", url));
    let (host, _) = url.split_once("/").unwrap_or((url, ""));
    match scheme.to_lowercase().as_str() {
        "rac" => Some((
            if host.contains(":") {
                host.to_string()
            } else {
                format!("{host}:42666")
            },
            false,
            false,
        )),
        "racs" => Some((
            if host.contains(":") {
                host.to_string()
            } else {
                format!("{host}:42667")
            },
            true,
            false,
        )),
        "wrac" => Some((
            if host.contains(":") {
                host.to_string()
            } else {
                format!("{host}:52666")
            },
            false,
            true,
        )),
        "wracs" => Some((
            if host.contains(":") {
                host.to_string()
            } else {
                format!("{host}:52667")
            },
            true,
            true,
        )),
        _ => None,
    }
}

/// Create RAC connection (also you can just TcpStream::connect)
///
/// host - host string, example: "example.com:12345", "example.com" (default port is 42666)
/// ssl - wrap with ssl client, write false if you dont know what it is
/// proxy - socks5 proxy (host, (user, pass))
/// wrac - to use wrac protocol
pub fn connect(host: &str, proxy: Option<String>) -> Result<RacStream, Box<dyn Error>> {
    let (host, ssl, wrac) =
        parse_rac_url(host).ok_or::<Box<dyn Error>>("url parse error".into())?;

    let stream: Box<dyn Stream> = if let Some(proxy) = proxy {
        if let Some((proxy, auth)) = parse_socks5_url(&proxy) {
            if let Some((user, pass)) = auth {
                Box::new(Socks5Stream::connect_with_password(
                    &proxy,
                    host.as_str(),
                    &user,
                    &pass,
                )?)
            } else {
                Box::new(Socks5Stream::connect(&proxy, host.as_str())?)
            }
        } else {
            return Err("proxy parse error".into());
        }
    } else {
        let addr = host
            .to_socket_addrs()?
            .next()
            .ok_or::<Box<dyn Error>>("addr parse error".into())?;

        Box::new(TcpStream::connect(&addr)?)
    };

    let stream = if ssl {
        let ip: String = host
            .split_once(":")
            .map(|o| o.0.to_string())
            .unwrap_or(host.clone());

        Box::new(
            TlsConnector::builder()
                .danger_accept_invalid_certs(true)
                .danger_accept_invalid_hostnames(true)
                .build()?
                .connect(&ip, stream)?,
        )
    } else {
        stream
    };

    stream.set_read_timeout(Duration::from_secs(15)); // TODO: softcode this
    stream.set_write_timeout(Duration::from_secs(15));

    if wrac {
        let (client, _) = client_with_config(
            &format!("ws://{host}"),
            stream,
            Some(WebSocketConfig::default().max_message_size(Some(512 * 1024 * 1024))), // TODO: softcode this
        )?;
        Ok(RacStream::WRAC(client))
    } else {
        Ok(RacStream::RAC(stream))
    }
}

/// Send message with fake auth
///
/// Explaination:
///
/// let (name, message) = message.split("> ") else { return send_message(stream, message) }
/// if send_message_auth(name, name, message) != 0 {
///     let name = "\x1f" + name
///     register_user(stream, name, name)
///     send_message_spoof_auth(stream, name + "> " + message)
/// }
pub fn send_message_spoof_auth(
    stream: &mut RacStream,
    message: &str,
) -> Result<(), Box<dyn Error>> {
    let Some((name, message)) = message.split_once("> ") else {
        return send_message(stream, message);
    };

    if let Ok(f) = send_message_auth(stream, &name, &name, &message) {
        if f != 0 {
            let name = format!("\x1f{name}");
            register_user(stream, &name, &name)?;
            send_message_spoof_auth(stream, &format!("{name}>  {message}"))?;
        }
    }

    Ok(())
}

/// Send message
///
/// stream - any stream that can be written to
/// message - message text
pub fn send_message(stream: &mut RacStream, message: &str) -> Result<(), Box<dyn Error>> {
    match stream {
        RacStream::WRAC(websocket) => wrac::send_message(websocket, message),
        RacStream::RAC(stream) => rac::send_message(stream, message),
    }
}

/// Register user
///
/// stream - any stream that can be written to
/// name - user name
/// password - user password
/// remove_null - remove null bytes on reading
///
/// returns whether the user was registered
pub fn register_user(
    stream: &mut RacStream,
    name: &str,
    password: &str,
) -> Result<bool, Box<dyn Error>> {
    match stream {
        RacStream::WRAC(websocket) => wrac::register_user(websocket, name, password),
        RacStream::RAC(stream) => rac::register_user(stream, name, password),
    }
}

/// Send message with auth
///
/// stream - any stream that can be written to
/// message - message text
/// name - user name
/// password - user password
/// remove_null - remove null bytes on reading
///
/// returns 0 if the message was sent successfully
/// returns 1 if the user does not exist
/// returns 2 if the password is incorrect
pub fn send_message_auth(
    stream: &mut RacStream,
    name: &str,
    password: &str,
    message: &str,
) -> Result<u8, Box<dyn Error>> {
    match stream {
        RacStream::WRAC(websocket) => wrac::send_message_auth(websocket, name, password, message),
        RacStream::RAC(stream) => rac::send_message_auth(stream, name, password, message),
    }
}

/// Read messages
///
/// max_messages - max messages in list
/// last_size - last returned packet size
/// remove_null - start with skipping null bytes
/// chunked - is chunked reading enabled
///
/// returns (messages, packet size)
pub fn read_messages(
    stream: &mut RacStream,
    max_messages: usize,
    last_size: usize,
    chunked: bool,
) -> Result<Option<(Vec<String>, usize)>, Box<dyn Error>> {
    match stream {
        RacStream::WRAC(websocket) => {
            wrac::read_messages(websocket, max_messages, last_size, chunked)
        }
        RacStream::RAC(stream) => rac::read_messages(stream, max_messages, last_size, chunked),
    }
}
