use std::{error::Error, io::{Read, Write}, net::TcpStream, sync::{atomic::Ordering, Arc}, thread, time::Duration};

use super::{term::print_console, Context, ADVERTISEMENT, ADVERTISEMENT_ENABLED};

pub fn send_message(context: Arc<Context>, message: &str) -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect(&context.host)?;
    stream.write_all(&[0x01])?;
    let data = format!("{}{}{}{}",
        if !context.disable_hiding_ip {
            "\r\x07"
        } else {
            ""
        },
        message,
        if !context.disable_hiding_ip && message.chars().count() < 39 { 
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

pub fn read_messages(context: Arc<Context>, last_size: usize) -> Result<Option<(Vec<String>, usize)>, Box<dyn Error>> {
    let mut stream = TcpStream::connect(&context.host)?;

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
        .skip(lines.len() - context.max_messages)
        .map(|o| o.to_string())
        .collect();

    Ok(Some((lines, packet_size)))
}

pub fn run_recv_loop(context: Arc<Context>) {
    thread::spawn({
        let cache = context.messages.clone();
        let update_time = context.update_time;
        let input = context.input.clone();

        move || {
            loop { 
                if let Ok(Some(data)) = read_messages(context.clone(), cache.1.load(Ordering::SeqCst)) {
                    *cache.0.write().unwrap() = data.0.clone();
                    cache.1.store(data.1, Ordering::SeqCst);
                    print_console(context.clone(), data.0, &input.read().unwrap()).expect("Error printing console");
                    thread::sleep(Duration::from_millis(update_time as u64));
                }
            }
        }
    });
}