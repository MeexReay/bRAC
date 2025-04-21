use std::{error::Error, io::{Read, Write}};
use tungstenite::{WebSocket, Message};


/// Send message
///
/// stream - any stream that can be written to
/// message - message text
pub fn send_message(
    stream: &mut WebSocket<impl Write + Read>,
    message: &str
) -> Result<(), Box<dyn Error>> {
    stream.write(Message::Binary(format!("\x01{message}").as_bytes().to_vec().into()))?;
    Ok(())
}

/// Register user
///
/// stream - any stream that can be written to
/// name - user name
/// password - user password
///
/// returns whether the user was registered
pub fn register_user(
    stream: &mut WebSocket<impl Write + Read>, 
    name: &str, 
    password: &str
) -> Result<bool, Box<dyn Error>> {
    stream.write(Message::Binary(format!("\x03{name}\n{password}").as_bytes().to_vec().into()))?;
    if let Ok(msg) = stream.read() {
        Ok(!msg.is_binary() || msg.into_data().get(0).unwrap_or(&0) == &0)
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
///
/// returns 0 if the message was sent successfully
/// returns 1 if the user does not exist
/// returns 2 if the password is incorrect
pub fn send_message_auth(
    stream: &mut WebSocket<impl Write + Read>, 
    name: &str, 
    password: &str, 
    message: &str
) -> Result<u8, Box<dyn Error>> {
    stream.write(Message::Binary(format!("\x02{name}\n{password}\n{message}").as_bytes().to_vec().into()))?;
    if let Ok(msg) = stream.read() {
        if msg.is_binary() {
            Ok(0)
        } else {
            Ok(*msg.into_data().get(0).unwrap_or(&0))
        }
    } else {
        Ok(0)
    }
}

/// Read messages
///
/// max_messages - max messages in list
/// last_size - last returned packet size
/// chunked - is chunked reading enabled
///
/// returns (messages, packet size)
pub fn read_messages(
    stream: &mut WebSocket<impl Write + Read>, 
    max_messages: usize, 
    last_size: usize, 
    chunked: bool
) -> Result<Option<(Vec<String>, usize)>, Box<dyn Error>> {
    stream.write(Message::Binary(vec![0x00].into()))?;

    let packet_size = {
        let msg = stream.read()?;
        if !msg.is_binary() {
            return Err("msg is not binary".into());
        }
        let len = msg.into_data().to_vec();

        String::from_utf8(len)?
            .trim_matches(char::from(0))
            .parse()?
    };

    if last_size == packet_size {
        return Ok(None);
    }

    let to_read = if !chunked || last_size == 0 {
        stream.write(Message::Binary(vec![0x00, 0x01].into()))?;
        packet_size
    } else {
        stream.write(Message::Binary(format!("\x00\x02{}", last_size).as_bytes().to_vec().into()))?;
        packet_size - last_size
    };

    let msg = stream.read()?;
    if !msg.is_binary() {
        return Err("msg is not binary".into());
    }
    let packet_data = msg.into_data().to_vec();

    if packet_data.len() > to_read {
        return Err("too big msg".into());
    }

    let packet_data = String::from_utf8_lossy(&packet_data).to_string();

    let lines: Vec<&str> = packet_data.split("\n").collect();
    let lines: Vec<String> = lines.clone().into_iter()
        .skip(if lines.len() >= max_messages { lines.len() - max_messages } else { 0 })
        .map(|o| o.to_string())
        .collect();

    Ok(Some((lines, packet_size)))
}