use std::{
    error::Error,
    io::{stdin, stdout, BufRead, Write}, 
    sync::{Arc, RwLock}, 
};

use colored::Color;
use rac::{run_recv_loop, send_message};
use rand::random;
use regex::Regex;
use lazy_static::lazy_static;
use term::run_main_loop;


const DEFAULT_HOST: &str = "meex.lol:11234";
const ADVERTISEMENT: &str = "\r\x1B[1A use bRAC client! https://github.com/MeexReay/bRAC \x1B[1B";

const MAX_MESSAGES: usize = 100;
const MAGIC_KEY: &str = "\u{B9AC}\u{3E70}";
const ADVERTISEMENT_ENABLED: bool = false;
const UPDATE_TIME: u64 = 50;


mod term;
mod rac;


lazy_static! {
    static ref DATE_REGEX: Regex = Regex::new(r"\[(.*?)\] (.*)").unwrap();
    static ref COLORED_USERNAMES: Vec<(Regex, Color)> = vec![
        (Regex::new(&format!(r"{}<(.*?)> (.*)", MAGIC_KEY)).unwrap(), Color::Green),
        (Regex::new(r"\u{2550}\u{2550}\u{2550}<(.*?)> (.*)").unwrap(), Color::BrightRed),
        (Regex::new(r"(.*?): (.*)").unwrap(), Color::Magenta),
        (Regex::new(r"<(.*?)> (.*)").unwrap(), Color::Cyan),
    ];
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

fn on_command(host: &str, command: &str) -> Result<(), Box<dyn Error>> {
    let command = command.trim_start_matches("/");
    let (command, args) = command.split_once(" ").unwrap_or((&command, ""));
    let args = args.split(" ").collect::<Vec<&str>>();

    if command == "clear" {
        send_message(host, &format!("\r\x1B[1A{}", " ".repeat(64)).repeat(MAX_MESSAGES))?;
        // *input.write().unwrap() = "/ заспамлено)))".to_string();
    } else if command == "spam" {
        send_message(host, &format!("\r\x1B[1A{}{}", args.join(" "), " ".repeat(10)).repeat(MAX_MESSAGES))?;
        // *input.write().unwrap() = "/ заспамлено)))".to_string();
    } else if command == "help" {
        write!(stdout(), "/clear - clear console; /spam *args - spam console with text; /help - show help message")?;
        stdout().flush()?;
    }

    Ok(())
}

fn main() {
    let host = get_input(&format!("Host (default: {}) > ", DEFAULT_HOST), DEFAULT_HOST);
    let anon_name = format!("Anon#{:X}", random::<u16>());
    let name = get_input(&format!("Name (default: {}) > ", anon_name), &anon_name);

    let messages = Arc::new(RwLock::new(String::new()));
    let input = Arc::new(RwLock::new(String::new()));

    run_recv_loop(host.clone(), messages.clone(), input.clone());
    run_main_loop(messages.clone(), input.clone(), host.clone(), name.clone());
}
