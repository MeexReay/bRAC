use std::{error::Error, io::{stdout, Write}, sync::{atomic::Ordering, Arc}, time::{Duration, SystemTime}};

use colored::{Color, Colorize};
use crossterm::{cursor::MoveLeft, event::{self, Event, KeyCode, KeyModifiers}, terminal::{disable_raw_mode, enable_raw_mode}, ExecutableCommand};
use rand::random;

use super::{proto::read_messages, util::sanitize_text, ADVERTISEMENT, COLORED_USERNAMES, DATE_REGEX, config::Context, proto::send_message};

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

pub fn print_console(context: Arc<Context>, messages: Vec<String>, input: &str) -> Result<(), Box<dyn Error>> {
    let text = format!(
        "{}{}\n> {}", 
        "\n".repeat(context.max_messages - messages.len()), 
        if context.disable_formatting {
            messages
        } else {
            messages.into_iter().filter_map(|o| format_message(context.clone(), o)).collect()
        }.join("\n"), 
        input
    );
    for line in text.lines() {
        write!(stdout().lock(), "\r\n{}", line)?;
        stdout().lock().flush()?;
    }
    Ok(())
}

fn format_message(ctx: Arc<Context>, message: String) -> Option<String> {
    let message = message.trim_end_matches(ADVERTISEMENT);
    let message = sanitize_text(&message);
    if ADVERTISEMENT.len() > 0 && 
        message.starts_with(ADVERTISEMENT
        .trim_start_matches("\r")
        .trim_start_matches("\n")) {
        return None
    }

    let date = DATE_REGEX.captures(&message)?;
    let (date, ip, message) = (
        date.get(1)?.as_str().to_string(), 
        date.get(2)?.as_str().to_string(), 
        date.get(3)?.as_str().to_string()
    );

    let prefix = if ctx.enable_ip_viewing {
        format!("{}{} [{}]", ip, " ".repeat(15-ip.len()), date)
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

fn poll_events(ctx: Arc<Context>) {
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
                        let message = ctx.input.read().unwrap().clone();
        
                        if !message.is_empty() {
                            let input_len = ctx.input.read().unwrap().chars().count();
                            write!(stdout(), 
                                "{}{}{}", 
                                MoveLeft(1).to_string().repeat(input_len), 
                                " ".repeat(input_len), 
                                MoveLeft(1).to_string().repeat(input_len)
                            ).unwrap();
                            stdout().lock().flush().unwrap();
                            ctx.input.write().unwrap().clear();

                            if message.starts_with("/") && !ctx.disable_commands {
                                on_command(ctx.clone(), &message).expect("Error on command");
                            } else {
                                let message = ctx.message_format.replace("{name}", &ctx.name).replace("{text}", &message);
                                send_message(ctx.clone(), &message).expect("Error sending message");
                            }
                        } else {
                            print_console(
                                ctx.clone(),
                                ctx.messages.0.read().unwrap().clone(), 
                                &ctx.input.read().unwrap()
                            ).expect("Error printing console");
                        }
                    }
                    KeyCode::Backspace => {
                        if ctx.input.write().unwrap().pop().is_some() {
                            stdout().lock().execute(MoveLeft(1)).unwrap();
                            write!(stdout(), " {}", MoveLeft(1).to_string()).unwrap();
                            stdout().lock().flush().unwrap();
                        }
                    }
                    KeyCode::Esc => {
                        disable_raw_mode().unwrap();
                        break;
                    }
                    KeyCode::Up => {
                        disable_raw_mode().unwrap();
                        break;
                    }
                    KeyCode::Down => {
                        disable_raw_mode().unwrap();
                        break;
                    }
                    KeyCode::PageUp => {
                        disable_raw_mode().unwrap();
                        break;
                    }
                    KeyCode::PageDown => {
                        disable_raw_mode().unwrap();
                        break;
                    }
                    KeyCode::Char(c) => {
                        if event.modifiers.contains(KeyModifiers::CONTROL) && "zxcZXCячсЯЧС".contains(c) {
                            disable_raw_mode().unwrap();
                            break;
                        }
                        ctx.input.write().unwrap().push(c);
                        write!(stdout(), "{}", c).unwrap();
                        stdout().lock().flush().unwrap();
                    }
                    _ => {}
                }
            },
            Event::Paste(data) => {
                ctx.input.write().unwrap().push_str(&data);
                write!(stdout(), "{}", &data).unwrap();
                stdout().lock().flush().unwrap();
            },
            Event::Mouse(data) => {
                
            }
            _ => {}
        }
    }
}

pub fn run_main_loop(ctx: Arc<Context>) {
    enable_raw_mode().unwrap();
    poll_events(ctx);
}