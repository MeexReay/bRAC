use std::{
    error::Error, io::{stdin, stdout, BufRead, Write}, sync::{atomic::AtomicUsize, Arc, RwLock}
};

use colored::Color;
use config::{get_config_path, load_config, Config};
use rac::{read_messages, run_recv_loop, send_message};
use rand::random;
use regex::Regex;
use lazy_static::lazy_static;
use term::run_main_loop;
use clap::Parser;


const ADVERTISEMENT: &str = "\r\x1B[1A use bRAC client! https://github.com/MeexReay/bRAC \x1B[1B";
const ADVERTISEMENT_ENABLED: bool = false;

lazy_static! {
    static ref DATE_REGEX: Regex = Regex::new(r"\[(.*?)\] (.*)").unwrap();
    static ref COLORED_USERNAMES: Vec<(Regex, Color)> = vec![
        (Regex::new(r"\u{B9AC}\u{3E70}<(.*?)> (.*)").unwrap(), Color::Green),
        (Regex::new(r"\u{2550}\u{2550}\u{2550}<(.*?)> (.*)").unwrap(), Color::BrightRed),
        (Regex::new(r"(.*?): (.*)").unwrap(), Color::Magenta),
        (Regex::new(r"<(.*?)> (.*)").unwrap(), Color::Cyan),
    ];
}


mod config;
mod term;
mod rac;


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

fn on_command(config: Arc<Config>, host: &str, disable_hiding_ip: bool, command: &str) -> Result<(), Box<dyn Error>> {
    let command = command.trim_start_matches("/");
    let (command, args) = command.split_once(" ").unwrap_or((&command, ""));
    let args = args.split(" ").collect::<Vec<&str>>();

    if command == "clear" {
        send_message(host, &format!("\r\x1B[1A{}", " ".repeat(64)).repeat(config.max_messages), disable_hiding_ip)?;
        // *input.write().unwrap() = "/ заспамлено)))".to_string();
    } else if command == "spam" {
        send_message(host, &format!("\r\x1B[1A{}{}", args.join(" "), " ".repeat(10)).repeat(config.max_messages), disable_hiding_ip)?;
        // *input.write().unwrap() = "/ заспамлено)))".to_string();
    } else if command == "help" {
        write!(stdout(), "Help message:\r
/clear - clear console\r
/spam *args - spam console with text\r
/help - show help message\r
\r
Press enter to close")?;
        stdout().flush()?;
    }

    Ok(())
}


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Print config path
    #[arg(short='p', long)]
    config_path: bool,

    /// Use specified host
    #[arg(short='H', long)]
    host: Option<String>,

    /// Use specified name
    #[arg(short, long)]
    name: Option<String>,

    /// Use specified message format
    #[arg(short='F', long)]
    message_format: Option<String>,

    /// Print unformatted messages from chat and exit
    #[arg(short, long)]
    read_messages: bool,

    /// Send unformatted message to chat and exit
    #[arg(short, long, value_name="MESSAGE")]
    send_message: Option<String>,

    /// Disable message formatting and sanitizing
    #[arg(short='f', long)]
    disable_formatting: bool,

    /// Disable slash commands
    #[arg(short='c', long)]
    disable_commands: bool,

    /// Disable ip hiding
    #[arg(short='i', long)]
    disable_ip_hiding: bool,
}


fn main() {
    let args = Args::parse();

    let config_path = get_config_path();

    if args.config_path {
        print!("{}", config_path.to_string_lossy());
        return;
    }
    
    // let start_time = SystemTime::now();
    let mut config = load_config(config_path);

    let name = match args.name.clone().or(config.name.clone()) {
        Some(i) => i,
        None => {
            let anon_name = format!("Anon#{:X}", random::<u16>());
            get_input(&format!("Name (default: {}) > ", anon_name)).unwrap_or(anon_name)
        },
    };

    if let Some(host) = args.host {
        config.host = host;
    }
    
    if let Some(message_format) = args.message_format {
        config.message_format = message_format;
    }

    let disable_hiding_ip = args.disable_ip_hiding;

    if let Some(message) = args.send_message {
        send_message(&config.host, &message, disable_hiding_ip).expect("Error sending message");
        return;
    }

    if args.read_messages {
        print!("{}", read_messages(&config.host, config.max_messages, 0).ok().flatten().expect("Error reading messages").0.join("\n"));
        return;
    }

    let disable_formatting = args.disable_formatting;
    let disable_commands = args.disable_commands;

    let messages = Arc::new((RwLock::new(Vec::new()), AtomicUsize::new(0)));
    let input = Arc::new(RwLock::new(String::new()));
    let config = Arc::new(config);



    // let elapsed = start_time.elapsed().unwrap().as_millis();
    // if elapsed < 1500 {
    //     thread::sleep(Duration::from_millis((1500 - elapsed) as u64));
    // }

    run_recv_loop(config.clone(), config.host.clone(), messages.clone(), input.clone(), disable_formatting);
    run_main_loop(config.clone(), messages.clone(), input.clone(), config.host.clone(), name.clone(), disable_formatting, disable_commands, disable_hiding_ip);
}
