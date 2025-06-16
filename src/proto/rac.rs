use std::{error::Error, io::{Read, Write}};

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
    password: &str
) -> Result<bool, Box<dyn Error>> {
    stream.write_all(format!("\x03{name}\n{password}").as_bytes())?;
    if let Ok(out) = skip_null(stream) {
        Ok(out[0] == 0)
    } else {
        Ok(true)
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
) -> Result<u8, Box<dyn Error>> {
    stream.write_all(format!("\x02{name}\n{password}\n{message}").as_bytes())?;
    if let Ok(out) = skip_null(stream) {
        Ok(out[0])
    } else {
        Ok(0)
    }
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
    chunked: bool
) -> Result<Option<(Vec<String>, usize)>, Box<dyn Error>> {
    stream.write_all(&[0x00])?;

    let packet_size = {
        let mut data = skip_null(stream)?;
        let mut buf = vec![0; 10];
        let len = stream.read(&mut buf)?;
        buf.truncate(len);
        data.append(&mut buf);
        remove_trailing_null(&mut data)?;

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

    let mut packet_data = skip_null(stream)?;
    let mut buf = vec![0; to_read - 1];
    stream.read_exact(&mut buf)?;
    packet_data.append(&mut buf);

    let packet_data = String::from_utf8_lossy(&packet_data).to_string();

    let lines: Vec<&str> = packet_data.split("\n").collect();
    let lines: Vec<String> = lines.clone().into_iter()
        .skip(if lines.len() >= max_messages { lines.len() - max_messages } else { 0 })
        .map(|o| o.to_string())
        .collect();

    Ok(Some((lines, packet_size)))
}