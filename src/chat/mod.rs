use std::{
    error::Error,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::connect_rac;

use super::proto::{connect, read_messages, register_user, send_message, send_message_auth};

use lazy_static::lazy_static;
use regex::Regex;

use ctx::Context;

#[cfg(feature = "gtk")]
pub mod gui;
#[cfg(feature = "gtk")]
pub use gui::run_main_loop;
#[cfg(feature = "gtk")]
use gui::{add_chat_messages, clear_chat_messages};

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
    pub static ref AVATAR_REGEX: Regex = Regex::new(r"(.*)\x06!!AR!!(.*)").unwrap();

    pub static ref DEFAULT_USER_AGENT: Regex = Regex::new(r"<(.*?)> (.*)").unwrap();

    pub static ref USER_AGENTS: Vec<(Regex, String)> = vec![
        (Regex::new(r"\u{B9AC}\u{3E70}<(.*?)> (.*)").unwrap(),         "#70fa7a".to_string()),     // bRAC
        (Regex::new(r"\u{2550}\u{2550}\u{2550}<(.*?)> (.*)").unwrap(), "#fa7070".to_string()),     // CRAB
        (Regex::new(r"\u{00B0}\u{0298}<(.*?)> (.*)").unwrap(),         "#da70fa".to_string()),     // Mefidroniy
        (Regex::new(r"\u{2042}<(.*?)> (.*)").unwrap(),                 "#f8b91b".to_string()),     // cRACk
        (Regex::new(r"\u{0D9E}<(.*?)> (.*)").unwrap(),                 "#aeff00".to_string()),     // Snowdrop
        (Regex::new(r"\u{30C4}<(.*?)> (.*)").unwrap(),                 "#ff5733".to_string()),     // Crack
        (Regex::new(r"<(.*?)> (.*)").unwrap(),                         "#70fadc".to_string()),     // clRAC
    ];

    pub static ref SERVER_LIST: Vec<String> = vec![
        "wracs://meex.lol:11234".to_string(),
        "rac://meex.lol".to_string(),
        "wracs://meex.lol".to_string(),
        "rac://91.192.22.20".to_string()
    ];
}

pub mod config;
pub mod ctx;

pub fn sanitize_text(input: &str) -> String {
    let without_ansi = ANSI_REGEX.replace_all(input, "");
    let cleaned_text = CONTROL_CHARS_REGEX.replace_all(&without_ansi, "");
    cleaned_text.into_owned()
}

#[cfg(feature = "gtk")]
pub fn add_message(ctx: Arc<Context>, message: &str) -> Result<(), Box<dyn Error>> {
    for i in message.split("\n").map(|o| o.to_string()) {
        print_message(ctx.clone(), i)?;
    }
    Ok(())
}

#[cfg(feature = "gtk")]
pub fn on_command(ctx: Arc<Context>, command: &str) -> Result<(), Box<dyn Error>> {
    let command = command.trim_start_matches("/");
    let (command, args) = command.split_once(" ").unwrap_or((&command, ""));
    let args = args.split(" ").collect::<Vec<&str>>();

    if command == "clear" {
        let Some(times) = args.get(0) else {
            return Ok(());
        };
        let times = times.parse()?;
        for _ in 0..times {
            send_message(connect_rac!(ctx), "\r")?;
        }
    } else if command == "spam" {
        let Some(times) = args.get(0) else {
            return Ok(());
        };
        let times = times.parse()?;
        let msg = args[1..].join(" ");
        for _ in 0..times {
            send_message(connect_rac!(ctx), &("\r".to_string() + &msg))?;
        }
    } else if command == "help" {
        add_message(ctx.clone(), HELP_MESSAGE)?;
    } else if command == "register" {
        let Some(pass) = args.get(0) else {
            add_message(ctx.clone(), "please provide password as the first argument")?;
            return Ok(());
        };

        match register_user(connect_rac!(ctx), &ctx.name(), pass) {
            Ok(true) => {
                add_message(ctx.clone(), "you was registered successfully bro")?;
                *ctx.registered.write().unwrap() = Some(pass.to_string());
            }
            Ok(false) => add_message(ctx.clone(), "user with this account already exists bruh")?,
            Err(e) => add_message(ctx.clone(), &format!("ERROR while registrationing: {}", e))?,
        };
    } else if command == "login" {
        let Some(pass) = args.get(0) else {
            add_message(ctx.clone(), "please provide password as the first argument")?;
            return Ok(());
        };

        add_message(ctx.clone(), "ye bro you was logged in")?;
        *ctx.registered.write().unwrap() = Some(pass.to_string());
    } else if command == "ping" {
        let mut before = ctx.packet_size();
        let message = format!(
            "Checking ping... {:X}",
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis()
        );

        send_message(connect_rac!(ctx), &message)?;

        let start = SystemTime::now();

        loop {
            let data = read_messages(
                connect_rac!(ctx),
                ctx.config(|o| o.max_messages),
                before,
                ctx.config(|o| o.chunked_enabled),
            )
            .ok()
            .flatten();

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

        add_message(
            ctx.clone(),
            &format!("Ping = {}ms", start.elapsed().unwrap().as_millis()),
        )?;
    } else {
        add_message(ctx.clone(), "Unknown command bruh")?;
    }

    Ok(())
}

pub fn prepare_message(ctx: Arc<Context>, message: &str) -> String {
    format!(
        "{}{}{}",
        if ctx.config(|o| o.hide_my_ip) {
            "\r\x07"
        } else {
            ""
        },
        message,
        if !ctx.config(|o| o.hide_my_ip) {
            if message.chars().count() < 54 {
                " ".repeat(54 - message.chars().count())
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    )
}

#[cfg(feature = "gtk")]
pub fn print_message(ctx: Arc<Context>, message: String) -> Result<(), Box<dyn Error>> {
    ctx.add_message(ctx.config(|o| o.max_messages), vec![message.clone()]);
    add_chat_messages(ctx.clone(), vec![message]);
    Ok(())
}

#[cfg(feature = "gtk")]
pub fn recv_tick(ctx: Arc<Context>) -> Result<(), Box<dyn Error>> {
    let last_size = ctx.packet_size();

    match read_messages(
        connect_rac!(ctx),
        ctx.config(|o| o.max_messages),
        ctx.packet_size(),
        ctx.config(|o| o.chunked_enabled),
    ) {
        Ok(Some((messages, size))) => {
            if ctx.config(|o| o.chunked_enabled) {
                ctx.add_messages_packet(ctx.config(|o| o.max_messages), messages.clone(), size);
                if last_size == 0 {
                    clear_chat_messages(ctx.clone(), messages);
                } else {
                    add_chat_messages(ctx.clone(), messages);
                }
            } else {
                ctx.put_messages_packet(ctx.config(|o| o.max_messages), messages.clone(), size);
                clear_chat_messages(ctx.clone(), messages);
            }
        }
        Err(e) => {
            if ctx.config(|o| o.debug_logs) {
                add_chat_messages(
                    ctx.clone(),
                    vec![format!("Read messages error: {}", e.to_string())],
                );
            }
        }
        _ => {}
    }

    Ok(())
}

#[cfg(feature = "gtk")]
pub fn on_send_message(ctx: Arc<Context>, message: &str) -> Result<(), Box<dyn Error>> {
    if message.starts_with("/") && ctx.config(|o| o.commands_enabled) {
        on_command(ctx.clone(), &message)?;
    } else {
        let mut message = prepare_message(
            ctx.clone(),
            &ctx.config(|o| o.message_format.clone())
                .replace("{name}", &ctx.name())
                .replace("{text}", &message),
        );

        if let Some(avatar) = ctx.config(|o| o.avatar.clone()) {
            message = format!("{message}\x06!!AR!!{avatar}"); // TODO: softcode this shittttttt
        }

        if let Some(password) = ctx.registered.read().unwrap().clone() {
            send_message_auth(connect_rac!(ctx), &ctx.name(), &password, &message)?;
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

/// message -> avatar
pub fn grab_avatar(message: &str) -> Option<String> {
    if let Some(message) = AVATAR_REGEX.captures(&message) {
        Some(message.get(2)?.as_str().to_string())
    } else {
        None
    }
}

/// message -> (date, ip, text, (name, color), avatar)
pub fn parse_message(
    message: String,
) -> Option<(
    String,
    Option<String>,
    String,
    Option<(String, String)>,
    Option<String>,
)> {
    if message.is_empty() {
        return None;
    }

    let (message, avatar) = if let Some(message) = AVATAR_REGEX.captures(&message) {
        (
            message.get(1)?.as_str().to_string(),
            Some(message.get(2)?.as_str().to_string()),
        )
    } else {
        (message, None)
    };

    let message = sanitize_message(message)?;

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
        (
            Some(message.get(1)?.as_str().to_string()),
            message.get(2)?.as_str().to_string(),
        )
    } else {
        (None, message)
    };

    let (message, nick) = if let Some((nick, message, color)) = DEFAULT_USER_AGENT
        .captures(&message)
        .and_then(|o| parse_user_agent(&o[2].to_string()))
    {
        (message, Some((nick, color)))
    } else if let Some((nick, message, color)) = parse_user_agent(&message) {
        (message, Some((nick, color)))
    } else {
        (message, None)
    };

    Some((date, ip, message, nick, avatar))
}

// message -> (nick, content, color)
pub fn parse_user_agent(message: &str) -> Option<(String, String, String)> {
    for (re, color) in USER_AGENTS.iter() {
        if let Some(captures) = re.captures(message) {
            return Some((
                captures[1].to_string(),
                captures[2].to_string(),
                color.clone(),
            ));
        }
    }
    None
}
