use std::{
    error::Error, io::{stdin, stdout, BufRead, Read, Write}, net::TcpStream, sync::{Arc, RwLock}, thread
};

use colored::Colorize;
use rand::random;
use regex::Regex;
use termion::{event::Key, input::TermRead, raw::IntoRawMode};

const MAX_MESSAGES: usize = 100;
const DEFAULT_HOST: &str = "meex.lol:11234";
const MAGIC_KEY: &str = "리㹰";

fn send_message(host: &str, message: &str) -> Result<(), Box<dyn Error>> {
    let mut stream = TcpStream::connect(host)?;
    stream.write_all(&[0x01])?;
    let data = format!("\r\x07{}{}", 
        message,
        if message.chars().count() < 39 { 
            " ".repeat(39-message.chars().count()) 
        } else { 
            String::new()
        }
    );
    stream.write_all(data.as_bytes())?;
    stream.write_all("\0".repeat(1023 - data.len()).as_bytes())?;
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

fn read_messages(host: &str) -> Result<String, Box<dyn Error>> {
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

    Ok(packet_data)
}

fn recv_loop(host: &str, cache: Arc<RwLock<String>>, input: Arc<RwLock<String>>) -> Result<(), Box<dyn Error>> {
    while let Ok(data) = read_messages(host) {
        if data == cache.read().unwrap().clone() { 
            continue 
        }

        *cache.write().unwrap() = data;
        print_console(&cache.read().unwrap(), &input.read().unwrap(), true)?;
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

fn sanitize_text(input: &str) -> String {
    let ansi_regex = Regex::new(r"\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])").unwrap();
    let control_chars_regex = Regex::new(r"[\x00-\x1F\x7F]").unwrap();
    let without_ansi = ansi_regex.replace_all(input, "");
    let cleaned_text = control_chars_regex.replace_all(&without_ansi, "");
    cleaned_text.into_owned()
}

fn on_message(message: String) -> String {
    let message = Regex::new(r"\{[^}]*\}\ ").unwrap().replace(&message, "").to_string();
    let message = sanitize_text(&message);

    if let Some(captures) = Regex::new(r"\[(.*?)\] <(.*?)> (.*)").unwrap().captures(&message) {
        let date = &captures[1];
        let nick = &captures[2];
        let content = &captures[3];

        format!(
            "{} {} {}",
            format!("[{}]", date).white().dimmed(),
            format!("<{}>", nick).cyan().bold(),
            content.white().blink()
        )
    } else if let Some(captures) = Regex::new(&format!(r"\[(.*?)\] {}<(.*?)> (.*)", MAGIC_KEY)).unwrap().captures(&message) {
        let date = &captures[1];
        let nick = &captures[2];
        let content = &captures[3];

        format!(
            "{} {} {}",
            format!("[{}]", date).white().dimmed(),
            format!("<{}>", nick).green().bold(),
            content.white().blink()
        )
    } else if let Some(captures) = Regex::new(r"\[(.*?)\] (.*?): (.*)").unwrap().captures(&message) {
        let date = &captures[1];
        let nick = &captures[2];
        let content = &captures[3];

        format!(
            "{} {} {}",
            format!("[{}]", date).white().dimmed(),
            format!("<{}>", nick).magenta().bold(),
            content.white().blink()
        )
    } else if let Some(captures) = Regex::new(r"\[(.*?)\] \u{2550}\u{2550}\u{2550}<(.*?)> (.*)").unwrap().captures(&message) {
        let date = &captures[1];
        let nick = &captures[2];
        let content = &captures[3];

        format!(
            "{} {} {}",
            format!("[{}]", date).white().dimmed(),
            format!("<{}>", nick).bright_red().bold(),
            content.white().blink()
        )
    } else if let Some(captures) = Regex::new(r"\[(.*?)\] (.*)").unwrap().captures(&message) {
        let date = &captures[1];
        let content = &captures[2];

        format!(
            "{} {}",
            format!("[{}]", date).white().dimmed(),
            content.white().blink()
        )
    } else {
        message.to_string()
    }
}

fn print_console(messages: &str, input: &str, sound: bool) -> Result<(), Box<dyn Error>> {
    let mut messages = messages.split("\n")
        .map(|o| o.to_string())
        .collect::<Vec<String>>();
    messages.reverse();
    messages.truncate(MAX_MESSAGES);
    messages.reverse();
    let messages: Vec<String> = messages.into_iter().map(on_message).collect();
    let mut out = stdout().into_raw_mode()?;
    let text = format!(
        "{}{}\n{}> {}", 
        "\n".repeat(MAX_MESSAGES - messages.len()), 
        messages.join("\n"), 
        if sound { "\x07" } else { "" }, 
        input
    );
    for line in text.lines() {
        write!(out, "\r\n{}", line)?;
        out.flush()?;
    }
    Ok(())
}

fn main() {
    let host = get_input(&format!("Host (default: {}) > ", DEFAULT_HOST), DEFAULT_HOST);
    let anon_name = format!("Anon#{:X}", random::<u16>());
    let name = get_input(&format!("Name (default: {}) > ", anon_name), &anon_name);

    let messages = Arc::new(RwLock::new(String::new()));
    let input = Arc::new(RwLock::new(String::new()));

    thread::spawn({
        let host = host.clone();
        let messages = messages.clone();
        let input = input.clone();

        move || {
            let _ = recv_loop(&host, messages, input);
            println!("Connection closed");
        }
    });

    let _ = stdout().into_raw_mode().unwrap();

    let stdin = stdin();
    for key in stdin.keys() {
        match key.unwrap() {
            Key::Char('\n') => {
                let message = input.read().unwrap().clone();
                if !message.is_empty() {
                    send_message(&host, &format!("{}<{}> {}", MAGIC_KEY, name, message)).expect("Error sending message");
                    input.write().unwrap().clear();
                }
                print_console(&messages.read().unwrap(), &input.read().unwrap(), false).expect("Error printing console");
            }
            Key::Backspace => {
                input.write().unwrap().pop();
            }
            Key::Char(c) => {
                input.write().unwrap().push(c);
            }
            Key::Esc => break,
            Key::Ctrl('c') => break,
            Key::Ctrl('z') => break,
            Key::Ctrl('x') => break,
            _ => {}
        }
    }
}
