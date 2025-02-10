use std::{
    error::Error, io::{stdin, stdout, BufRead, Write}, sync::{Arc, RwLock}, thread, time::{Duration, SystemTime, UNIX_EPOCH} 
};

use colored::Color;
use config::{get_config_path, load_config, Config};
use rac::{run_recv_loop, send_message};
use rand::random;
use regex::Regex;
use lazy_static::lazy_static;
use term::run_main_loop;


const ADVERTISEMENT: &str = "\r\x1B[1A use bRAC client! https://github.com/MeexReay/bRAC \x1B[1B";
const ADVERTISEMENT_ENABLED: bool = false;


mod config;
mod term;
mod rac;


lazy_static! {
    static ref DATE_REGEX: Regex = Regex::new(r"\[(.*?)\] (.*)").unwrap();
    static ref COLORED_USERNAMES: Vec<(Regex, Color)> = vec![
        (Regex::new(r"\u{B9AC}\u{3E70}<(.*?)> (.*)").unwrap(), Color::Green),
        (Regex::new(r"\u{2550}\u{2550}\u{2550}<(.*?)> (.*)").unwrap(), Color::BrightRed),
        (Regex::new(r"(.*?): (.*)").unwrap(), Color::Magenta),
        (Regex::new(r"<(.*?)> (.*)").unwrap(), Color::Cyan),
    ];
}



fn get_input(prompt: &str) -> Option<String> {
    let mut out = stdout().lock();
    out.write_all(prompt.as_bytes()).ok()?;
    out.flush().ok()?;
    let input = stdin().lock().lines().next()
        .map(|o| o.ok())
        .flatten()?;

    if input.is_empty() { 
        None 
    } else { 
        Some(input.to_string())
    }
}


fn on_command(config: Arc<Config>, host: &str, command: &str) -> Result<(), Box<dyn Error>> {
    let command = command.trim_start_matches("/");
    let (command, args) = command.split_once(" ").unwrap_or((&command, ""));
    let args = args.split(" ").collect::<Vec<&str>>();

    if command == "clear" {
        send_message(host, &format!("\r\x1B[1A{}", " ".repeat(64)).repeat(config.max_messages))?;
        // *input.write().unwrap() = "/ заспамлено)))".to_string();
    } else if command == "spam" {
        send_message(host, &format!("\r\x1B[1A{}{}", args.join(" "), " ".repeat(10)).repeat(config.max_messages))?;
        // *input.write().unwrap() = "/ заспамлено)))".to_string();
    } else if command == "help" {
        write!(stdout(), "/clear - clear console; /spam *args - spam console with text; /help - show help message")?;
        stdout().flush()?;
    }

    Ok(())
}

fn main() {
    let start_time = SystemTime::now();
    let config = load_config(get_config_path());

    let name = match config.name.clone() {
        Some(i) => i,
        None => {
            let anon_name = format!("Anon#{:X}", random::<u16>());
            get_input(&format!("Name (default: {}) > ", anon_name)).unwrap_or(anon_name)
        },
    };

    let messages = Arc::new(RwLock::new(String::new()));
    let input = Arc::new(RwLock::new(String::new()));
    let config = Arc::new(config);

    let elapsed = start_time.elapsed().unwrap().as_millis();
    if elapsed < 1500 {
        thread::sleep(Duration::from_millis((1500 - elapsed) as u64));
    }

    run_recv_loop(config.clone(), config.host.clone(), messages.clone(), input.clone());
    run_main_loop(config.clone(), messages.clone(), input.clone(), config.host.clone(), name.clone());
}
