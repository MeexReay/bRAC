use std::{error::Error, fmt::format, io::{stdin, stdout, BufRead, Read, Write}, net::TcpStream, thread, time::{self, SystemTime, UNIX_EPOCH}};

const MAX_MESSAGES: usize = 100;
const DEFAULT_HOST: &str = "meex.lol:11234";

fn send_message(host: &str, message: &str) -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect(host)?;
    stream.write_all(&[0x01])?;
    stream.write_all(message.as_bytes())?;
    stream.write_all("\0".repeat(1023 - message.len()).as_bytes())?;
    Ok(())
}

fn read_messages(host: &str, skip: usize) -> Result<String, Box<dyn Error>> {
    let mut stream = TcpStream::connect(host)?;

    stream.write_all(&[0x00])?;

    let packet_size = {
        let mut data = Vec::new();
        
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

    // println!("{} {}", skip, packet_size);

    if packet_size <= skip {
        return Ok(String::new())
    }

    let to_read = if skip == 0 {
        stream.write_all(&[0x01])?;
        packet_size
    } else {
        stream.write_all(&[0x02])?;
        stream.write_all(skip.to_string().as_bytes())?;
        packet_size - skip
    };

    let packet_data = {
        let mut data = vec![0; to_read];
        stream.read_exact(&mut data)?;
        data.retain(|x| *x != 0);
        while String::from_utf8_lossy(&data).len() != to_read {
            let mut buf = vec![0; to_read - data.len()];
            stream.read_exact(&mut buf)?;
            data.append(&mut buf);
            data.retain(|x| *x != 0);
        }
        String::from_utf8_lossy(&data).to_string()
    };

    // println!("{}", packet_data);

    Ok(packet_data)
}

fn print_console(messages: Vec<&str>) -> Result<(), Box<dyn Error>> {
    let mut messages = messages.clone();
    messages.reverse();
    messages.truncate(MAX_MESSAGES);
    messages.reverse();
    let mut out = stdout().lock();
    let text = format!("{}{}\n> ", "\n".repeat(MAX_MESSAGES - messages.len()), messages.join("\n"));
    out.write_all(text.as_bytes())?;
    out.flush()?;
    Ok(())
}

fn recv_loop(host: &str) -> Result<(), Box<dyn Error>> {
    let mut cache = String::new();
    while let Ok(messages) = read_messages(host, cache.len()) {
        if messages.len() == 0 { continue }
        cache.push_str(&messages);
        print_console(cache.split("\n").collect())?;
    }
    Ok(())
}

fn get_input(prompt: &str, default: &str) -> String {
    let input = || -> Option<String> {
        let mut out = stdout().lock();
        out.write_all(prompt.as_bytes()).ok()?;
        out.flush().ok()?;
        stdin().lock().lines().next()
            .map(|o| o.ok())
            .flatten()
    }();

    if let Some(input) = &input {
        if input.is_empty() { 
            default 
        } else { 
            input
        }
    } else { 
        default 
    }.to_string()
}

fn main() {
    let host = get_input(&format!("Host (default: {}) > ", DEFAULT_HOST), DEFAULT_HOST);

    let anon_name = format!("Anon#{:X}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());

    let name = get_input(&format!("Name (default: {}) > ", anon_name), &anon_name);

    thread::spawn({
        let host = host.clone();

        move || {
            let _ = recv_loop(&host);
            println!("Connection closed");
        }
    });

    let mut lines = stdin().lock().lines();
    while let Some(Ok(message)) = lines.next() {
        send_message(&host, &format!("<{}> {}", &name, &message)).expect("Error sending message");
    }
}
