use std::{
    error::Error,
    sync::{atomic::{AtomicUsize, Ordering}, Arc, RwLock}, 
    time::{SystemTime, UNIX_EPOCH}
};

use crate::proto::{register_user, send_message_auth};

use super::{
    proto::{connect, read_messages, send_message, send_message_spoof_auth}, 
    util::sanitize_text
};

use lazy_static::lazy_static;
use regex::Regex;

use ctx::Context;

pub use gui::{
    ChatContext,
    print_message,
    run_main_loop
};


lazy_static! {
  pub static ref DATE_REGEX: Regex = Regex::new(r"\[(.*?)\] (.*)").unwrap();
  pub static ref IP_REGEX: Regex = Regex::new(r"\{(.*?)\} (.*)").unwrap();

  pub static ref COLORED_USERNAMES: Vec<(Regex, String)> = vec![
      (Regex::new(r"\u{B9AC}\u{3E70}<(.*?)> (.*)").unwrap(),         "green".to_string()),     // bRAC
      (Regex::new(r"\u{2550}\u{2550}\u{2550}<(.*?)> (.*)").unwrap(), "red".to_string()), // CRAB
      (Regex::new(r"\u{00B0}\u{0298}<(.*?)> (.*)").unwrap(),         "magenta".to_string()),   // Mefidroniy
      (Regex::new(r"<(.*?)> (.*)").unwrap(),                         "cyan".to_string()),      // clRAC
  ];
}


pub mod gui;
pub mod config;
pub mod ctx;


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
    for i in message.split("\n")
        .map(|o| o.to_string()) {
        print_message(ctx.clone(), i)?;
    }
    Ok(())
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
                *ctx.chat().registered.write().unwrap() = Some(pass.to_string());
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
        *ctx.chat().registered.write().unwrap() = Some(pass.to_string());
    } else if command == "ping" {
        let mut before = ctx.chat().messages.packet_size();
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

        if let Some(password) = ctx.chat().registered.read().unwrap().clone() {
            send_message_auth(&mut connect(&ctx.host, ctx.enable_ssl)?, &ctx.name, &password, &message)?;
        } else if ctx.enable_auth {
            send_message_spoof_auth(&mut connect(&ctx.host, ctx.enable_ssl)?, &message)?;
        } else {
            send_message(&mut connect(&ctx.host, ctx.enable_ssl)?, &message)?;
        }
    }

    Ok(())
} 

/// message -> (date, ip, text, (name, color))
pub fn parse_message(message: String) -> Option<(String, Option<String>, String, Option<(String, String)>)> {
    let message = sanitize_text(&message);

    let message = message
        .trim_start_matches("(UNREGISTERED)")
        .trim_start_matches("(UNAUTHORIZED)")
        .trim_start_matches("(UNAUTHENTICATED)")
        .trim()
        .to_string()+" ";

    if message.is_empty() {
        return None
    }
    
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
    
    let (message, nick) = match find_username_color(&message) {
        Some((name, content, color)) => (content, Some((name, color))),
        None => (message, None),
    };

    Some((date, ip, message, nick))
}

// message -> (nick, content, color)
pub fn find_username_color(message: &str) -> Option<(String, String, String)> {
    for (re, color) in COLORED_USERNAMES.iter() {
        if let Some(captures) = re.captures(message) {
            return Some((captures[1].to_string(), captures[2].to_string(), color.clone()))
        }
    }
    None
}

pub fn set_chat(ctx: Arc<Context>, chat: ChatContext) {
    *ctx.chat.write().unwrap() = Some(Arc::new(chat));
}