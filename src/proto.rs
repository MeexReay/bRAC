#![allow(unused)]

use std::{error::Error, fmt::Debug, io::{Read, Write}, net::{SocketAddr, TcpStream, ToSocketAddrs}, str::FromStr, time::Duration};
use native_tls::{TlsConnector, TlsStream};
use socks::Socks5Stream;

use crate::util::parse_socks5_url;

pub trait RacStream: Read + Write + Unpin + Send + Sync + Debug {
    fn set_read_timeout(&self, timeout: Duration);
    fn set_write_timeout(&self, timeout: Duration);
}

impl RacStream for TcpStream {
    fn set_read_timeout(&self, timeout: Duration) { let _ = TcpStream::set_read_timeout(&self, Some(timeout)); }
    fn set_write_timeout(&self, timeout: Duration) { let _ = TcpStream::set_write_timeout(&self, Some(timeout)); }
}

impl RacStream for Socks5Stream {
    fn set_read_timeout(&self, timeout: Duration) { let _ = TcpStream::set_read_timeout(self.get_ref(), Some(timeout)); }
    fn set_write_timeout(&self, timeout: Duration) { let _ = TcpStream::set_write_timeout(self.get_ref(), Some(timeout)); }
}

impl<T: RacStream> RacStream for TlsStream<T> {
    fn set_read_timeout(&self, timeout: Duration) { self.get_ref().set_read_timeout(timeout); }
    fn set_write_timeout(&self, timeout: Duration) { self.get_ref().set_write_timeout(timeout); }
}

impl RacStream for TlsStream<Box<dyn RacStream>> {
    fn set_read_timeout(&self, timeout: Duration) { self.get_ref().set_read_timeout(timeout); }
    fn set_write_timeout(&self, timeout: Duration) { self.get_ref().set_write_timeout(timeout); }
}

/// Create RAC connection (also you can just TcpStream::connect)
///
/// host - host string, example: "example.com:12345", "example.com" (default port is 42666)
/// ssl - wrap with ssl client, write false if you dont know what it is
/// proxy - socks5 proxy (host, (user, pass))
pub fn connect(host: &str, ssl: bool, proxy: Option<String>) -> Result<Box<dyn RacStream>, Box<dyn Error>> {
    let host = if host.contains(":") {
        host.to_string()
    } else {
        format!("{host}:42666")
    };

    let stream: Box<dyn RacStream> = if let Some(proxy) = proxy {
        if let Some((proxy, auth)) = parse_socks5_url(&proxy) {
            if let Some((user, pass)) = auth {
                Box::new(Socks5Stream::connect_with_password(&proxy, host.as_str(), &user, &pass)?)
            } else {
                Box::new(Socks5Stream::connect(&proxy, host.as_str())?)
            }
        } else {
            return Err("proxy parse error".into());
        }
    } else {
        let addr = host.to_socket_addrs()?.next().ok_or::<Box<dyn Error>>("addr parse error".into())?;

        Box::new(TcpStream::connect(&addr)?)
    };

    let stream = if ssl {
        let ip: String = host.split_once(":")
            .map(|o| o.0.to_string())
            .unwrap_or(host.clone());

        Box::new(TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true)
            .build()?
            .connect(&ip, stream)?)
    } else {
        stream
    };

    stream.set_read_timeout(Duration::from_secs(3));
    stream.set_write_timeout(Duration::from_secs(3));

    Ok(stream)
}

/// Send message
///
/// stream - any stream that can be written to
/// message - message text
pub fn send_message(stream: &mut impl Write, message: &str) -> Result<(), Box<dyn Error>> {
    stream.write_all(format!("\x01{message}").as_bytes())?;
    Ok(())
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
    stream: &mut (impl Write + Read), 
    name: &str, 
    password: &str, 
    remove_null: bool
) -> Result<bool, Box<dyn Error>> {
    stream.write_all(format!("\x03{name}\n{password}").as_bytes())?;
    if remove_null {
        if let Ok(out) = skip_null(stream) {
            Ok(out[0] == 0)
        } else {
            Ok(true)
        }
    } else {
        let mut buf = vec![0];
        if let Ok(1) = stream.read(&mut buf) {
            Ok(buf[0] == 0)
        } else {
            Ok(true)
        }
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
    stream: &mut (impl Write + Read), 
    name: &str, 
    password: &str, 
    message: &str, 
    remove_null: bool
) -> Result<u8, Box<dyn Error>> {
    stream.write_all(format!("\x02{name}\n{password}\n{message}").as_bytes())?;

    if remove_null {
        if let Ok(out) = skip_null(stream) {
            Ok(out[0])
        } else {
            Ok(0)
        }
    } else {
        let mut buf = vec![0];
        if let Ok(1) = stream.read(&mut buf) {
            Ok(buf[0])
        } else {
            Ok(0)
        }
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
pub fn send_message_spoof_auth(stream: &mut (impl Write + Read), message: &str, remove_null: bool) -> Result<(), Box<dyn Error>> {
    let Some((name, message)) = message.split_once("> ") else { return send_message(stream, message) };

    if let Ok(f) = send_message_auth(stream, &name, &message, &message, remove_null) {
        if f != 0 {
            let name = format!("\x1f{name}");
            register_user(stream, &name, &name, remove_null);
            send_message_spoof_auth(stream, &format!("{name}>  {message}"), remove_null);
        }
    }

    Ok(())
}

/// Skip null bytes and return first non-null byte
pub fn skip_null(stream: &mut impl Read) -> Result<Vec<u8>, Box<dyn Error>> {
    loop {
        let mut buf = vec![0; 1];
        stream.read_exact(&mut buf)?;
        if buf[0] != 0 {
            break Ok(buf)
        }
    }
}

/// remove trailing null bytes in vector
pub fn remove_trailing_null(vec: &mut Vec<u8>) -> Result<(), Box<dyn Error>> {
    while vec.ends_with(&[0]) {
        vec.remove(vec.len()-1);
    }
    Ok(())
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
    stream: &mut (impl Read + Write), 
    max_messages: usize, 
    last_size: usize, 
    remove_null: bool, 
    chunked: bool
) -> Result<Option<(Vec<String>, usize)>, Box<dyn Error>> {
    stream.write_all(&[0x00])?;

    let packet_size = {
        let data = if remove_null {
            let mut data = skip_null(stream)?;
            let mut buf = vec![0; 10];
            let len = stream.read(&mut buf)?;
            buf.truncate(len);
            data.append(&mut buf);
            remove_trailing_null(&mut data)?;
            data
        } else {
            let mut data = vec![0; 10];
            let len = stream.read(&mut data)?;
            data.truncate(len);
            data
        };

        String::from_utf8(data)?
            .trim_matches(char::from(0))
            .parse()?
    };

    if last_size == packet_size {
        return Ok(None);
    }

    let to_read = if !chunked || last_size == 0 {
        stream.write_all(&[0x01])?;
        packet_size
    } else {
        stream.write_all(format!("\x02{}", last_size).as_bytes())?;
        packet_size - last_size
    };

    let packet_data = if remove_null {
        let mut data = skip_null(stream)?;
        let mut buf = vec![0; to_read - 1];
        stream.read_exact(&mut buf)?;
        data.append(&mut buf);
        data
    } else {
        let mut data = vec![0; to_read];
        stream.read_exact(&mut data)?;
        data
    };

    let packet_data = String::from_utf8_lossy(&packet_data).to_string();

    let lines: Vec<&str> = packet_data.split("\n").collect();
    let lines: Vec<String> = lines.clone().into_iter()
        .skip(if lines.len() >= max_messages { lines.len() - max_messages } else { 0 })
        .map(|o| o.to_string())
        .collect();

    Ok(Some((lines, packet_size)))
}