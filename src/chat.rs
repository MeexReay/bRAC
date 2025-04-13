use std::{
    error::Error, io::{stdout, Write}, 
    sync::{atomic::{AtomicUsize, Ordering}, Arc, RwLock}, 
    time::{SystemTime, UNIX_EPOCH}
};

use colored::{Color, Colorize};

use super::{
    proto::{connect, read_messages, send_message, send_message_spoof_auth}, 
    IP_REGEX, 
    util::sanitize_text, 
    COLORED_USERNAMES, 
    DATE_REGEX, 
    config::Context
};

#[cfg(not(feature = "pretty"))]
pub mod minimal_tui;
#[cfg(not(feature = "pretty"))]
pub use minimal_tui::run_main_loop;

#[cfg(feature = "pretty")]
pub mod pretty_tui;
#[cfg(feature = "pretty")]
pub use pretty_tui::run_main_loop;


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

    pub fn update(&self, messages: Vec<String>, packet_size: usize) {
        self.packet_size.store(packet_size, Ordering::SeqCst);
        *self.messages.write().unwrap() = messages;
    }

    pub fn append_and_store(&self, messages: Vec<String>, packet_size: usize) {
        self.packet_size.store(packet_size, Ordering::SeqCst);
        self.messages.write().unwrap().append(&mut messages.clone());
    }

    pub fn append(&self, messages: Vec<String>) {
        self.messages.write().unwrap().append(&mut messages.clone());
    }
}


const HELP_MESSAGE: &str = "Help message:\r
/help - show help message\r
/clear n - send empty message n times\r
/spam n text - send message with text n times\r
/ping - check server ping\r
\r
Press enter to close";


pub fn on_command(ctx: Arc<Context>, command: &str) -> Result<(), Box<dyn Error>> {
    let command = command.trim_start_matches("/");
    let (command, args) = command.split_once(" ").unwrap_or((&command, ""));
    let args = args.split(" ").collect::<Vec<&str>>();

    let response = move || -> Option<String> { 
        if command == "clear" {
            let times = args.get(0)?.parse().ok()?;
            for _ in 0..times {
                send_message(&mut connect(&ctx.host, ctx.enable_ssl).ok()?, "\r").ok()?;
            }
            None
        } else if command == "spam" {
            let times = args.get(0)?.parse().ok()?;
            let msg = args[1..].join(" ");
            for _ in 0..times {
                send_message(&mut connect(&ctx.host, ctx.enable_ssl).ok()?, &("\r".to_string()+&msg)).ok()?;
            }
            None
            // send_message(&mut connect(&ctx.host, ctx.enable_ssl)?, 
            //     &prepare_message(ctx.clone(), 
            //         &format!("\r\x1B[1A{}{}", args.join(" "), " ".repeat(10)).repeat(ctx.max_messages)
            //         ))?;
        } else if command == "help" {
            Some(HELP_MESSAGE.to_string())
        } else if command == "ping" {
            let mut before = ctx.messages.packet_size();
            let message = format!("Checking ping... {:X}", SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_millis());

            send_message(&mut connect(&ctx.host, ctx.enable_ssl).ok()?, &message).ok()?;

            let start = SystemTime::now();

            loop {
                let data = read_messages(
                    &mut connect(&ctx.host, ctx.enable_ssl).ok()?, 
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

            send_message(&mut connect(&ctx.host, ctx.enable_ssl).ok()?, &format!("Ping = {}ms", start.elapsed().unwrap().as_millis())).ok()?;

            None
        } else {
            None
        }
    }();

    if let Some(response) = response {
        write!(stdout(), "{}", response)?;
        stdout().flush()?;
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

        if ctx.enable_auth {
            send_message_spoof_auth(&mut connect(&ctx.host, ctx.enable_ssl)?, &message)?;
        } else {
            send_message(&mut connect(&ctx.host, ctx.enable_ssl)?, &message)?;
        }
    }

    Ok(())
} 

pub fn format_message(ctx: Arc<Context>, message: String) -> Option<String> {
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

    let prefix = if ctx.enable_ip_viewing {
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
}

fn find_username_color(message: &str) -> Option<(String, String, Color)> {
    for (re, color) in COLORED_USERNAMES.iter() {
        if let Some(captures) = re.captures(message) {
            return Some((captures[1].to_string(), captures[2].to_string(), color.clone()))
        }
    }
    None
}