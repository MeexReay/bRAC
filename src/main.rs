use std::{
    error::Error, io::{stdin, stdout, BufRead, Write}, sync::{atomic::{AtomicUsize, Ordering}, Arc, RwLock}, time::SystemTime
};

use colored::Color;
use config::{get_config_path, load_config};
use rac::{read_messages, run_recv_loop, send_message};
use rand::random;
use regex::Regex;
use lazy_static::lazy_static;
use term::run_main_loop;
use clap::Parser;


const ADVERTISEMENT: &str = "\r\x1B[1A use bRAC client! https://github.com/MeexReay/bRAC \x1B[1B";
const ADVERTISEMENT_ENABLED: bool = false;

lazy_static! {
    static ref DATE_REGEX: Regex = Regex::new(r"\[(.*?)\] \{(.*?)\} (.*)").unwrap();
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

fn on_command(ctx: Arc<Context>, command: &str) -> Result<(), Box<dyn Error>> {
    let command = command.trim_start_matches("/");
    let (command, args) = command.split_once(" ").unwrap_or((&command, ""));
    let args = args.split(" ").collect::<Vec<&str>>();

    if command == "clear" {
        send_message(ctx.clone(), &format!("\r\x1B[1A{}", " ".repeat(64)).repeat(ctx.max_messages))?;
    } else if command == "spam" {
        send_message(ctx.clone(), &format!("\r\x1B[1A{}{}", args.join(" "), " ".repeat(10)).repeat(ctx.max_messages))?;
    } else if command == "help" {
        write!(stdout(), "Help message:\r
/help - show help message\r
/clear - clear console\r
/spam *args - spam console with text\r
/ping - check server ping\r
\r
Press enter to close")?;
        stdout().flush()?;
    } else if command == "ping" {
        let mut before = ctx.messages.1.load(Ordering::SeqCst);
        let start = SystemTime::now();
        let message = format!("Checking ping... {:X}", random::<u16>());
        send_message(ctx.clone(), &message)?;
        loop {
            let data = read_messages(ctx.clone(), before).ok().flatten();

            if let Some((data, size)) = data {
                if let Some(last) = data.iter().rev().find(|o| o.contains(&message)) {
                    if last.contains(&message) {
                        break;
                    } else {
                        before = size;
                    }
                } else {
                    before = size;
                }
            }
        }
        send_message(ctx.clone(), &format!("Ping = {}ms", start.elapsed().unwrap().as_millis()))?;
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

    /// Enable users IP viewing
    #[arg(short='v', long)]
    enable_users_ip_viewing: bool,
}


struct Context {
    messages: Arc<(RwLock<Vec<String>>, AtomicUsize)>, 
    input: Arc<RwLock<String>>,
    host: String, 
    name: String, 
    disable_formatting: bool, 
    disable_commands: bool, 
    disable_hiding_ip: bool,
    message_format: String,
    update_time: usize,
    max_messages: usize,
    enable_ip_viewing: bool
}


fn main() {
    let args = Args::parse();
    
    let context = {
        let config_path = get_config_path();
    
        if args.config_path {
            print!("{}", config_path.to_string_lossy());
            return;
        }

        let config = load_config(config_path);

        Context {
            messages: Arc::new((RwLock::new(Vec::new()), AtomicUsize::new(0))), 
            input: Arc::new(RwLock::new(String::new())),

            message_format: args.message_format.clone().unwrap_or(config.message_format.clone()), 
            host: args.host.clone().unwrap_or(config.host.clone()), 
            name: match args.name.clone().or(config.name.clone()) {
                Some(i) => i,
                None => {
                    let anon_name = format!("Anon#{:X}", random::<u16>());
                    get_input(&format!("Name (default: {}) > ", anon_name)).unwrap_or(anon_name)
                },
            }, 
            disable_formatting: args.disable_formatting, 
            disable_commands: args.disable_commands, 
            disable_hiding_ip: args.disable_ip_hiding,
            update_time: config.update_time,
            max_messages: config.max_messages,
            enable_ip_viewing: args.enable_users_ip_viewing || config.enable_ip_viewing
        }
    };

    let context = Arc::new(context);

    if args.read_messages {
        print!("{}", read_messages(context.clone(), 0).ok().flatten().expect("Error reading messages").0.join("\n"));
    }

    if let Some(message) = &args.send_message {
        send_message(context.clone(), message).expect("Error sending message");
    }

    if args.send_message.is_some() || args.read_messages {
        return;
    }

    run_recv_loop(context.clone());
    run_main_loop(context.clone());
}
