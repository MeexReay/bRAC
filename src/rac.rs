use std::{error::Error, io::{Read, Write}, net::TcpStream, sync::{atomic::{AtomicUsize, Ordering}, Arc, RwLock}, thread, time::Duration};

use crate::{config::Config, term::print_console, ADVERTISEMENT, ADVERTISEMENT_ENABLED};

pub fn send_message(host: &str, message: &str, disable_hiding_ip: bool) -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect(host)?;
    stream.write_all(&[0x01])?;
    let data = format!("{}{}{}{}",
        if !disable_hiding_ip {
            "\r\x07"
        } else {
            ""
        },
        message,
        if !disable_hiding_ip && message.chars().count() < 39 { 
            " ".repeat(39-message.chars().count()) 
        } else { 
            String::new()
        },
        if ADVERTISEMENT_ENABLED {ADVERTISEMENT} else {""}
    );
    stream.write_all(data.as_bytes())?;
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

fn recv_loop(config: Arc<Config>, host: &str, cache: Arc<(RwLock<Vec<String>>, AtomicUsize)>, input: Arc<RwLock<String>>, disable_formatting: bool) -> Result<(), Box<dyn Error>> {
    while let Ok(data) = read_messages(host, config.max_messages, cache.1.load(Ordering::SeqCst)) {
        if let Some(data) = data {
            *cache.0.write().unwrap() = data.0.clone();
            cache.1.store(data.1, Ordering::SeqCst);
            print_console(config.clone(), data.0, &input.read().unwrap(), disable_formatting)?;
            thread::sleep(Duration::from_millis(config.update_time as u64));
        }
    }
    Ok(())
}

pub fn run_recv_loop(config: Arc<Config>, host: String, messages: Arc<(RwLock<Vec<String>>, AtomicUsize)>, input: Arc<RwLock<String>>, disable_formatting: bool) {
    thread::spawn({
        move || {
            let _ = recv_loop(config.clone(), &host, messages, input, disable_formatting);
            println!("Connection closed");
        }
    });
}