use std::{
    error::Error,
    sync::{atomic::{AtomicUsize, Ordering}, Arc, RwLock}, 
    time::{SystemTime, UNIX_EPOCH}
};

use colored::{Color, Colorize};

use crate::proto::{register_user, send_message_auth};

use super::{
    proto::{connect, read_messages, send_message, send_message_spoof_auth}, 
    util::sanitize_text, 
    config::Context
};

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
  pub static ref DATE_REGEX: Regex = Regex::new(r"\[(.*?)\] (.*)").unwrap();
  pub static ref IP_REGEX: Regex = Regex::new(r"\{(.*?)\} (.*)").unwrap();

  pub static ref COLORED_USERNAMES: Vec<(Regex, Color)> = vec![
      (Regex::new(r"\u{B9AC}\u{3E70}<(.*?)> (.*)").unwrap(),            Color::Green),     // bRAC
      (Regex::new(r"\u{2550}\u{2550}\u{2550}<(.*?)> (.*)").unwrap(),    Color::BrightRed), // CRAB
      (Regex::new(r"\u{00B0}\u{0298}<(.*?)> (.*)").unwrap(),            Color::Magenta),   // Mefidroniy
      (Regex::new(r"<(.*?)> (.*)").unwrap(),                            Color::Cyan),      // clRAC
  ];
}

#[cfg(not(feature = "pretty"))]
pub mod minimal_tui;
#[cfg(not(feature = "pretty"))]
pub use minimal_tui::{run_main_loop, update_console};

#[cfg(feature = "pretty")]
pub mod pretty_tui;
#[cfg(feature = "pretty")]
pub use pretty_tui::{run_main_loop, update_console};


pub struct ChatStorage {
    messages: RwLock<Vec<String>>,
    packet_size: AtomicUsize
}

impl ChatStorage {
    pub fn new() -> Self {
        ChatStorage {
            messages: RwLock::new(Vec::new()),
            packet_size: AtomicUsize::default()
        }
    }

    pub fn packet_size(&self) -> usize {
        self.packet_size.load(Ordering::SeqCst)
    }

    pub fn messages(&self) -> Vec<String> {
        self.messages.read().unwrap().clone()
    }

    pub fn update(&self, max_length: usize, messages: Vec<String>, packet_size: usize) {
        self.packet_size.store(packet_size, Ordering::SeqCst);
        let mut messages = messages;
        if messages.len() > max_length {
            messages.drain(max_length..);
        }
        *self.messages.write().unwrap() = messages;
    }

    pub fn append_and_store(&self, max_length: usize, messages: Vec<String>, packet_size: usize) {
        self.packet_size.store(packet_size, Ordering::SeqCst);
        self.append(max_length, messages);
    }

    pub fn append(&self, max_length: usize, messages: Vec<String>) {
        self.messages.write().unwrap().append(&mut messages.clone());
        if self.messages.read().unwrap().len() > max_length {
            self.messages.write().unwrap().drain(max_length..);
        }
    }
}


const HELP_MESSAGE: &str = "Help message:
/help - show help message
/register password - register user
/login password - login user
/clear n - send empty message n times
/spam n text - send message with text n times
/ping - check server ping";


pub fn add_message(ctx: Arc<Context>, message: &str) -> Result<(), Box<dyn Error>> {
    ctx.messages.append(
        ctx.max_messages, 
        message.split("\n").map(|o| o.to_string()).collect::<Vec<String>>()
    );
    update_console(ctx)
}

pub fn on_command(ctx: Arc<Context>, command: &str) -> Result<(), Box<dyn Error>> {
    let command = command.trim_start_matches("/");
    let (command, args) = command.split_once(" ").unwrap_or((&command, ""));
    let args = args.split(" ").collect::<Vec<&str>>();

    if command == "clear" {
        let Some(times) = args.get(0) else { return Ok(()) };
        let times = times.parse()?;
        for _ in 0..times {
            send_message(&mut connect(&ctx.host, ctx.enable_ssl)?, "\r")?;
        }
    } else if command == "spam" {
        let Some(times) = args.get(0) else { return Ok(()) };
        let times = times.parse()?;
        let msg = args[1..].join(" ");
        for _ in 0..times {
            send_message(&mut connect(&ctx.host, ctx.enable_ssl)?, &("\r".to_string()+&msg))?;
        }
    } else if command == "help" {
        add_message(ctx.clone(), HELP_MESSAGE)?;
    } else if command == "register" {
        let Some(pass) = args.get(0) else { 
            add_message(ctx.clone(), "please provide password as the first argument")?;
            return Ok(()) 
        };

        match register_user(&mut connect(&ctx.host, ctx.enable_ssl)?, &ctx.name, pass) {
            Ok(true) => {
                add_message(ctx.clone(), "you was registered successfully bro")?;
                *ctx.registered.write().unwrap() = Some(pass.to_string());
            },
            Ok(false) => add_message(ctx.clone(), "user with this account already exists bruh")?,
            Err(e) => add_message(ctx.clone(), &format!("ERROR while registrationing: {}", e))?
        };
    } else if command == "login" {
        let Some(pass) = args.get(0) else { 
            add_message(ctx.clone(), "please provide password as the first argument")?;
            return Ok(()) 
        };

        add_message(ctx.clone(), "ye bro you was logged in")?;
        *ctx.registered.write().unwrap() = Some(pass.to_string());
    } else if command == "ping" {
        let mut before = ctx.messages.packet_size();
        let message = format!("Checking ping... {:X}", SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis());

        send_message(&mut connect(&ctx.host, ctx.enable_ssl)?, &message)?;

        let start = SystemTime::now();

        loop {
            let data = read_messages(
                &mut connect(&ctx.host, ctx.enable_ssl)?, 
                ctx.max_messages, 
                before, 
                !ctx.enable_ssl,
                ctx.enable_chunked
            ).ok().flatten();

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

        add_message(ctx.clone(), &format!("Ping = {}ms", start.elapsed().unwrap().as_millis()))?;
    } else {
        add_message(ctx.clone(), "Unknown command bruh")?;
    }

    Ok(())
}

pub fn prepare_message(context: Arc<Context>, message: &str) -> String {
    format!("{}{}{}",
        if !context.disable_hiding_ip {
            "\r\x07"
        } else {
            ""
        },
        message,
        if !context.disable_hiding_ip { 
            let spaces = if context.enable_auth {
                39
            } else {
                54
            };

            if message.chars().count() < spaces { 
                " ".repeat(spaces-message.chars().count()) 
            } else { 
                String::new()
            }
        } else {
            String::new()
        }
    )
}

pub fn on_send_message(ctx: Arc<Context>, message: &str) -> Result<(), Box<dyn Error>> {
    if message.starts_with("/") && !ctx.disable_commands {
        on_command(ctx.clone(), &message)?;
    } else {
        let message = prepare_message(
        ctx.clone(), 
        &ctx.message_format
            .replace("{name}", &ctx.name)
            .replace("{text}", &message)
        );

        if let Some(password) = ctx.registered.read().unwrap().clone() {
            send_message_auth(&mut connect(&ctx.host, ctx.enable_ssl)?, &ctx.name, &password, &message)?;
        } else if ctx.enable_auth {
            send_message_spoof_auth(&mut connect(&ctx.host, ctx.enable_ssl)?, &message)?;
        } else {
            send_message(&mut connect(&ctx.host, ctx.enable_ssl)?, &message)?;
        }
    }

    Ok(())
} 

pub fn format_message(enable_ip_viewing: bool, message: String) -> Option<String> {
    if message.is_empty() {
        None
    } else {
        Some(
        {
            let message = message.clone();
            move || -> Option<String> {
                let message = sanitize_text(&message);

                let date = DATE_REGEX.captures(&message)?;
                let (date, message) = (
                    date.get(1)?.as_str().to_string(), 
                    date.get(2)?.as_str().to_string(), 
                );

                let (ip, message) = if let Some(message) = IP_REGEX.captures(&message) {
                    (Some(message.get(1)?.as_str().to_string()), message.get(2)?.as_str().to_string())
                } else {
                    (None, message)
                };

                let message = message
                    .trim_start_matches("(UNREGISTERED)")
                    .trim_start_matches("(UNAUTHORIZED)")
                    .trim_start_matches("(UNAUTHENTICATED)")
                    .trim()
                    .to_string()+" ";

                let prefix = if enable_ip_viewing {
                    if let Some(ip) = ip {
                        format!("{}{} [{}]", ip, " ".repeat(if 15 >= ip.chars().count() {15-ip.chars().count()} else {0}), date)
                    } else {
                        format!("{} [{}]", " ".repeat(15), date)
                    }
                } else {
                    format!("[{}]", date)
                };

                Some(if let Some(captures) = find_username_color(&message) {
                    let nick = captures.0;
                    let content = captures.1;
                    let color = captures.2;

                        format!(
                        "{} {} {}",
                        prefix.white().dimmed(),
                        format!("<{}>", nick).color(color).bold(),
                        content.white().blink()
                    )
                } else {
                    format!(
                        "{} {}",
                        prefix.white().dimmed(),
                        message.white().blink()
                    )
                })
            }()
        }.unwrap_or_else(|| {
            format!(
                "{}",
                message.bright_white()
            )
        }))
    }
}

pub fn find_username_color(message: &str) -> Option<(String, String, Color)> {
    for (re, color) in COLORED_USERNAMES.iter() {
        if let Some(captures) = re.captures(message) {
            return Some((captures[1].to_string(), captures[2].to_string(), color.clone()))
        }
    }
    None
}