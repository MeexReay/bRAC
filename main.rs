use std::{error::Error, io::{stdin, stdout, BufRead, Read, Write}, net::TcpStream, thread};

const MAX_MESSAGES: usize = 100;
const DEFAULT_HOST: &str = "meex.lol:11234";

fn send_message(host: &str, message: &str) -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect(host)?;
    stream.write_all(&[0x01])?;
    stream.write_all(message.as_bytes())?;
    Ok(())
}

fn read_messages(host: &str) -> Result<Vec<String>, Box<dyn Error>> {
    let mut stream = TcpStream::connect(host)?;
    stream.write_all(&[0x00])?;
    let packet_size = {
        let mut buf= vec![0; 10];
        stream.read(&mut buf)?;
        String::from_utf8(buf)?.trim_matches(char::from(0)).parse()?
    };
    stream.write_all(&[0x01])?;
    let packet_data = {
        let mut buf = vec![0; packet_size];
        stream.read_exact(&mut buf)?;
        let buf_str = String::from_utf8_lossy(&buf).to_string();
        let start_null = buf_str.len() - buf_str.trim_start_matches(char::from(0)).len();
        let mut buf = vec![0; start_null];
        stream.read_exact(&mut buf)?;
        format!("{}{}", &buf_str, String::from_utf8_lossy(&buf).to_string())
    };
    let mut lines: Vec<String> = packet_data.split("\n").map(|o| o.to_string()).collect();
    lines.reverse();
    lines.truncate(MAX_MESSAGES);
    lines.reverse();
    Ok(lines)
}

fn print_console(messages: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut out = stdout().lock();
    let text = format!("{}{}\n> ", "\n".repeat(MAX_MESSAGES - messages.len()), messages.join("\n"));
    out.write_all(text.as_bytes())?;
    out.flush()?;
    Ok(())
}

fn recv_loop(host: &str) -> Result<(), Box<dyn Error>> {
    let mut cache = Vec::new();
    while let Ok(messages) = read_messages(host) {
        if cache == messages { continue }
        print_console(messages.clone())?;
        cache = messages;
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
    let prefix = get_input("Prefix (default: none) > ", "");

    thread::spawn({
        let host = host.clone();

        move || {
            let _ = recv_loop(&host);
            println!("Connection closed");
        }
    });

    let mut lines = stdin().lock().lines();
    while let Some(Ok(message)) = lines.next() {
        send_message(&host, &format!("{}{}", &prefix, &message)).expect("Error sending message");
    }
}
