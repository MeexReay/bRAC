use std::{error::Error, io::{Read, Write}, net::{Shutdown, TcpStream}};



pub fn send_message(host: &str, message: &str) -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect(host)?;
    stream.write_all(&[0x01])?;
    stream.write_all(message.as_bytes())?;
    Ok(())
}

pub fn send_message_auth(host: &str, message: &str) -> Result<(), Box<dyn Error>> {
    let Some((name, message)) = message.split_once("> ") else { return send_message(host, message) };

    let mut stream = TcpStream::connect(host)?;
    stream.write_all(&[0x02])?;
    stream.write_all(name.as_bytes())?;
    stream.write_all(b"\n")?;
    stream.write_all(name.as_bytes())?;
    stream.write_all(b"\n")?;
    stream.write_all(message.as_bytes())?;

    let mut buf = vec![0; 1];
    if let Ok(_) = stream.read_exact(&mut buf) {
        let name = format!("\x1f{name}");
        register_user(host, &name, &name)?;
        let message = format!("{name}> {message}");
        send_message_auth(host, &message)
    } else {
        Ok(())
    }
}

pub fn register_user(host: &str, name: &str, password: &str) -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect(host)?;
    stream.write_all(&[0x03])?;
    stream.write_all(name.as_bytes())?;
    stream.write_all(&[b'\n'])?;
    stream.write_all(password.as_bytes())?;
    Ok(())
}

fn skip_null(stream: &mut TcpStream) -> Result<Vec<u8>, Box<dyn Error>> {
    loop {
        let mut buf = vec![0; 1];
        stream.read_exact(&mut buf)?;
        if buf[0] != 0 {
            break Ok(buf)
        }
    }
}

pub fn read_messages(host: &str, max_messages: usize, last_size: usize) -> Result<Option<(Vec<String>, usize)>, Box<dyn Error>> {
    let mut stream = TcpStream::connect(host)?;

    stream.write_all(&[0x00])?;

    let packet_size = {
        let mut data = skip_null(&mut stream)?;
        
        loop {
            let mut buf = vec![0; 1];
            stream.read_exact(&mut buf)?;
            let ch = buf[0];
            if ch == 0 {
                break
            }
            data.push(ch);
        }

        String::from_utf8(data)?
            .trim_matches(char::from(0))
            .parse()?
    };

    if last_size == packet_size {
        return Ok(None);
    }

    stream.write_all(&[0x01])?;

    let packet_data = {
        let mut data = skip_null(&mut stream)?;
        while data.len() < packet_size {
            let mut buf = vec![0; packet_size - data.len()];
            let read_bytes = stream.read(&mut buf)?;
            buf.truncate(read_bytes);
            data.append(&mut buf);
        }
        String::from_utf8_lossy(&data).to_string()
    };

    let lines: Vec<&str> = packet_data.split("\n").collect();
    let lines: Vec<String> = lines.clone().into_iter()
        .skip(lines.len() - max_messages)
        .map(|o| o.to_string())
        .collect();

    Ok(Some((lines, packet_size)))
}