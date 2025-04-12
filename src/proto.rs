use std::{error::Error, fmt::Debug, io::{Read, Write}, net::TcpStream};

use native_tls::TlsConnector;

pub trait RacStream: Read + Write + Unpin + Send + Sync + Debug {}
impl<T: Read + Write + Unpin + Send + Sync + Debug> RacStream for T {}

pub fn connect(host: &str, ssl: bool) -> Result<Box<dyn RacStream>, Box<dyn Error>> {
    let host = if host.contains(":") {
        host.to_string()
    } else {
        format!("{host}:42666")
    };

    Ok(if ssl {
        let ip: String = host.split_once(":")
            .map(|o| o.0.to_string())
            .unwrap_or(host.clone());

        Box::new(TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true)
            .build()?
            .connect(&ip, connect(&host, false)?)?)
    } else {
        Box::new(TcpStream::connect(host)?)
    })
}

pub fn send_message(stream: &mut (impl Read + Write), message: &str) -> Result<(), Box<dyn Error>> {
    stream.write_all(format!("\x01{message}").as_bytes())?;
    Ok(())
}

pub fn send_message_auth(stream: &mut (impl Read + Write), message: &str) -> Result<(), Box<dyn Error>> {
    let Some((name, message)) = message.split_once("> ") else { return send_message(stream, message) };

    stream.write_all(format!("\x02{name}\n{name}\n{message}").as_bytes())?;

    let mut buf = vec![0; 1];
    if let Ok(_) = stream.read_exact(&mut buf) {
        let name = format!("\x1f{name}");
        register_user(stream, &name, &name)?;
        let message = format!("{name}> {message}");
        send_message_auth(stream, &message)
    } else {
        Ok(())
    }
}

pub fn register_user(stream: &mut (impl Read + Write), name: &str, password: &str) -> Result<(), Box<dyn Error>> {
    stream.write_all(format!("\x03{name}\n{password}").as_bytes())?;
    Ok(())
}

fn skip_null(stream: &mut impl Read) -> Result<Vec<u8>, Box<dyn Error>> {
    loop {
        let mut buf = vec![0; 1];
        stream.read_exact(&mut buf)?;
        if buf[0] != 0 {
            break Ok(buf)
        }
    }
}

pub fn read_messages(
    stream: &mut (impl Read + Write), 
    max_messages: usize, 
    last_size: usize, 
    start_null: bool, 
    chunked: bool
) -> Result<Option<(Vec<String>, usize)>, Box<dyn Error>> {
    stream.write_all(&[0x00])?;

    let packet_size = {
        let data = if start_null {
            let mut data = skip_null(stream)?;
            
            loop {
                let mut buf = vec![0; 1];
                stream.read_exact(&mut buf)?;
                let ch = buf[0];
                if ch == 0 {
                    break
                }
                data.push(ch);
            }
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

    let packet_data = {
        let data = if start_null {
            let mut data = skip_null(stream)?;
            while data.len() < to_read {
                let mut buf = vec![0; to_read - data.len()];
                stream.read_exact(&mut buf)?;
                data.append(&mut buf);
            }
            data
        } else {
            let mut data = vec![0; to_read];
            stream.read_exact(&mut data)?;
            data
        };

        String::from_utf8_lossy(&data).to_string()
    };

    let lines: Vec<&str> = packet_data.split("\n").collect();
    let lines: Vec<String> = lines.clone().into_iter()
        .skip(if lines.len() >= max_messages { lines.len() - max_messages } else { 0 })
        .map(|o| o.to_string())
        .collect();

    Ok(Some((lines, packet_size)))
}