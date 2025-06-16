use std::{
    error::Error, sync::Arc, time::{SystemTime, UNIX_EPOCH}
};

use crate::connect_rac;

use super::proto::{connect, read_messages, send_message, send_message_spoof_auth, register_user, send_message_auth};

use gui::{add_chat_messages, clear_chat_messages};
use lazy_static::lazy_static;
use regex::Regex;

use ctx::Context;

pub use gui::run_main_loop;


const HELP_MESSAGE: &str = "Help message:
/help - show help message
/register password - register user
/login password - login user
/clear n - send empty message n times
/spam n text - send message with text n times
/ping - check server ping";

lazy_static! {
    static ref ANSI_REGEX: Regex = Regex::new(r"\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])").unwrap();
    static ref CONTROL_CHARS_REGEX: Regex = Regex::new(r"[\x00-\x1F\x7F]").unwrap();

    pub static ref DATE_REGEX: Regex = Regex::new(r"\[(.*?)\] (.*)").unwrap();
    pub static ref IP_REGEX: Regex = Regex::new(r"\{(.*?)\} (.*)").unwrap();

    pub static ref COLORED_USERNAMES: Vec<(Regex, String)> = vec![
        (Regex::new(r"\u{B9AC}\u{3E70}<(.*?)> (.*)").unwrap(),         "#70fa7a".to_string()),     // bRAC
        (Regex::new(r"\u{2550}\u{2550}\u{2550}<(.*?)> (.*)").unwrap(), "#fa7070".to_string()),       // CRAB
        (Regex::new(r"\u{00B0}\u{0298}<(.*?)> (.*)").unwrap(),         "#da70fa".to_string()),   // Mefidroniy
        (Regex::new(r"<(.*?)> (.*)").unwrap(),                         "#70fadc".to_string()),      // clRAC
    ];

    pub static ref SERVER_LIST: Vec<String> = vec![
        "rac://meex.lol".to_string(), 
        "rac://meex.lol:11234".to_string(), 
        "rac://91.192.22.20".to_string()
    ];
}


pub mod gui;
pub mod config;
pub mod ctx;

pub fn sanitize_text(input: &str) -> String {
    let without_ansi = ANSI_REGEX.replace_all(input, "");
    let cleaned_text = CONTROL_CHARS_REGEX.replace_all(&without_ansi, "");
    cleaned_text.into_owned()
}

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
            send_message(connect_rac!(ctx), "\r")?;
        }
    } else if command == "spam" {
        let Some(times) = args.get(0) else { return Ok(()) };
        let times = times.parse()?;
        let msg = args[1..].join(" ");
        for _ in 0..times {
            send_message(connect_rac!(ctx), &("\r".to_string()+&msg))?;
        }
    } else if command == "help" {
        add_message(ctx.clone(), HELP_MESSAGE)?;
    } else if command == "register" {
        let Some(pass) = args.get(0) else { 
            add_message(ctx.clone(), "please provide password as the first argument")?;
            return Ok(()) 
        };

        match register_user(connect_rac!(ctx), &ctx.name(), pass) {
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
        let mut before = ctx.packet_size();
        let message = format!("Checking ping... {:X}", SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis());

        send_message(connect_rac!(ctx), &message)?;

        let start = SystemTime::now();

        loop {
            let data = read_messages(
                connect_rac!(ctx), 
                ctx.config(|o| o.max_messages), 
                before, 
                ctx.config(|o| o.chunked_enabled)
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

pub fn prepare_message(ctx: Arc<Context>, message: &str) -> String {
    format!("{}{}{}",
        if ctx.config(|o| o.hide_my_ip) {
            "\r\x07"
        } else {
            ""
        },
        message,
        if !ctx.config(|o| o.hide_my_ip) { 
            if message.chars().count() < 54 { 
                " ".repeat(54-message.chars().count()) 
            } else { 
                String::new()
            }
        } else {
            String::new()
        }
    )
}

pub fn print_message(ctx: Arc<Context>, message: String) -> Result<(), Box<dyn Error>> {
    ctx.add_message(ctx.config(|o| o.max_messages), vec![message.clone()]);
    add_chat_messages(ctx.clone(), vec![message]);
    Ok(())
}

pub fn recv_tick(ctx: Arc<Context>) -> Result<(), Box<dyn Error>> {
    let last_size = ctx.packet_size();

    match read_messages(
        connect_rac!(ctx), 
        ctx.config(|o| o.max_messages), 
        ctx.packet_size(), 
        ctx.config(|o| o.chunked_enabled)
    ) {
        Ok(Some((messages, size))) => {
            if ctx.config(|o| o.chunked_enabled) {
                ctx.add_messages_packet(ctx.config(|o| o.max_messages), messages.clone(), size);
                if last_size == 0 {
                    if messages.len() >= 1 {
                        clear_chat_messages(ctx.clone(), messages);
                    }
                } else {
                    add_chat_messages(ctx.clone(), messages);
                }
            } else {
                ctx.put_messages_packet(ctx.config(|o| o.max_messages), messages.clone(), size);
                clear_chat_messages(ctx.clone(), messages);
            }
        },
        Err(e) => {
            if ctx.config(|o| o.debug_logs) {
                add_chat_messages(ctx.clone(), vec![format!("Read messages error: {}", e.to_string())]);
            }
        }
        _ => {}
    }

    Ok(())
}

pub fn on_send_message(ctx: Arc<Context>, message: &str) -> Result<(), Box<dyn Error>> {
    if message.starts_with("/") && ctx.config(|o| o.commands_enabled) {
        on_command(ctx.clone(), &message)?;
    } else {
        let message = prepare_message(
        ctx.clone(), 
        &ctx.config(|o| o.message_format.clone())
            .replace("{name}", &ctx.name())
            .replace("{text}", &message)
        );

        if let Some(password) = ctx.registered.read().unwrap().clone() {
            send_message_auth(connect_rac!(ctx), &ctx.name(), &password, &message)?;
        } else if ctx.config(|o| o.auth_enabled) {
            send_message_spoof_auth(connect_rac!(ctx), &message)?;
        } else {
            send_message(connect_rac!(ctx), &message)?;
        }
    }

    Ok(())
} 

pub fn sanitize_message(message: String) -> Option<String> {
    let message = sanitize_text(&message);

    let message = message.trim().to_string();

    Some(message)
}

/// message -> (date, ip, text, (name, color))
pub fn parse_message(message: String) -> Option<(String, Option<String>, String, Option<(String, String)>)> {
    if message.is_empty() {
        return None
    }
    
    let date = DATE_REGEX.captures(&message)?;
    let (date, message) = (
        date.get(1)?.as_str().to_string(), 
        date.get(2)?.as_str().to_string(), 
    );

    let message = message
        .trim_start_matches("(UNREGISTERED)")
        .trim_start_matches("(UNAUTHORIZED)")
        .trim_start_matches("(UNAUTHENTICATED)")
        .trim()
        .to_string();

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
