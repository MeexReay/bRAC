use std::{error::Error, io::{stdout, Write}, sync::{Arc, RwLock}, thread, time::Duration};

use colored::{Color, Colorize};
use crossterm::{cursor::MoveLeft, event::{self, Event, KeyCode}, terminal::{disable_raw_mode, enable_raw_mode}, ExecutableCommand};
use regex::Regex;

use crate::{on_command, rac::send_message, ADVERTISEMENT, COLORED_USERNAMES, DATE_REGEX, MAGIC_KEY, MAX_MESSAGES};

pub fn print_console(messages: &str, input: &str) -> Result<(), Box<dyn Error>> {
    let mut messages = messages.split("\n")
        .map(|o| o.to_string())
        .collect::<Vec<String>>();
    messages.reverse();
    messages.truncate(MAX_MESSAGES);
    messages.reverse();
    let messages: Vec<String> = messages.into_iter().filter_map(format_message).collect();
    let text = format!(
        "{}{}\n> {}", 
        "\n".repeat(MAX_MESSAGES - messages.len()), 
        messages.join("\n"), 
        // if sound { "\x07" } else { "" }, 
        input
    );
    for line in text.lines() {
        write!(stdout().lock(), "\r\n{}", line)?;
        stdout().lock().flush()?;
    }
    Ok(())
}

fn format_message(message: String) -> Option<String> {
    let message = message.trim_end_matches(ADVERTISEMENT);
    let message = Regex::new(r"\{[^}]*\}\ ").unwrap().replace(&message, "").to_string();
    let message = sanitize_text(&message);
    if ADVERTISEMENT.len() > 0 && 
        message.starts_with(ADVERTISEMENT
        .trim_start_matches("\r")
        .trim_start_matches("\n")) {
        return None
    }

    let date = DATE_REGEX.captures(&message)?;
    let (date, message) = (date.get(1)?.as_str().to_string(), date.get(2)?.as_str().to_string());

    Some(if let Some(captures) = find_username_color(&message) {
        let nick = captures.0;
        let content = captures.1;
        let color = captures.2;

        format!(
            "{} {} {}",
            format!("[{}]", date).white().dimmed(),
            format!("<{}>", nick).color(color).bold(),
            content.white().blink()
        )
    } else {
        format!(
            "{} {}",
            format!("[{}]", date).white().dimmed(),
            message.white().blink()
        )
    })
}

fn sanitize_text(input: &str) -> String {
    let ansi_regex = Regex::new(r"\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])").unwrap();
    let control_chars_regex = Regex::new(r"[\x00-\x1F\x7F]").unwrap();
    let without_ansi = ansi_regex.replace_all(input, "");
    let cleaned_text = control_chars_regex.replace_all(&without_ansi, "");
    cleaned_text.into_owned()
}

/// nick content nick_color
fn find_username_color(message: &str) -> Option<(String, String, Color)> {
    for (re, color) in COLORED_USERNAMES.iter() {
        if let Some(captures) = re.captures(message) {
            return Some((captures[1].to_string(), captures[2].to_string(), color.clone()))
        }
    }
    None
}

fn poll_events(input: Arc<RwLock<String>>, messages: Arc<RwLock<String>>, host: String, name: String) {
    loop {
        if !event::poll(Duration::from_millis(50)).unwrap_or(false) { continue }

        let event = match event::read() {
            Ok(i) => i,
            Err(_) => { continue },
        };

        match event {
            Event::Key(event) => {
                match event.code {
                    KeyCode::Enter => {
                        let message = input.read().unwrap().clone();
        
                        if !message.is_empty() {
                            let input_len = input.read().unwrap().chars().count();
                            write!(stdout(), 
                                "{}{}{}", 
                                MoveLeft(1).to_string().repeat(input_len), 
                                " ".repeat(input_len), 
                                MoveLeft(1).to_string().repeat(input_len)
                            ).unwrap();
                            stdout().lock().flush().unwrap();
                            input.write().unwrap().clear();

                            if message.starts_with("/") {
                                on_command(&host, &message).expect("Error on command");
                            } else {
                                send_message(&host, &format!("{}<{}> {}", MAGIC_KEY, name, message)).expect("Error sending message");
                            }
                        } else {
                            print_console(
                                &messages.read().unwrap(), 
                                &input.read().unwrap()
                            ).expect("Error printing console");
                        }
                    }
                    KeyCode::Backspace => {
                        if input.write().unwrap().pop().is_some() {
                            stdout().lock().execute(MoveLeft(1)).unwrap();
                            write!(stdout(), " {}", MoveLeft(1).to_string()).unwrap();
                            stdout().lock().flush().unwrap();
                        }
                    }
                    KeyCode::Char(c) => {
                        input.write().unwrap().push(c);
                        write!(stdout(), "{}", c).unwrap();
                        stdout().lock().flush().unwrap();
                    }
                    KeyCode::Esc => {
                        disable_raw_mode().unwrap();
                        break;
                    },
                    _ => {}
                }
            },
            Event::Paste(data) => {
                input.write().unwrap().push_str(&data);
                write!(stdout(), "{}", &data).unwrap();
                stdout().lock().flush().unwrap();
            }
            _ => {}
        }
    }
}

pub fn run_main_loop(messages: Arc<RwLock<String>>, input: Arc<RwLock<String>>, host: String, name: String) {
    enable_raw_mode().unwrap();

    // thread::spawn({
    //     let messages = messages.clone();
    //     let input = input.clone();

    //     move || {
    //         loop {
    //             print_console(
    //                 &messages.read().unwrap(), 
    //                 &input.read().unwrap()
    //             ).expect("Error printing console");
    //             thread::sleep(Duration::from_secs(5));
    //         }
    //     }
    // });
    
    poll_events(input.clone(), messages.clone(), host, name);
}