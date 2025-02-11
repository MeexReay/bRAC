use std::{cmp::{max, min}, error::Error, io::{stdout, Write}, sync::{atomic::{AtomicUsize, Ordering}, Arc, RwLock}, thread, time::{Duration, SystemTime}};

use colored::{Color, Colorize};
use crossterm::{cursor::{MoveLeft, MoveRight}, event::{self, Event, KeyCode, KeyModifiers, MouseEventKind}, terminal::{self, disable_raw_mode, enable_raw_mode}};
use rand::random;

use crate::{util::string_chunks, IP_REGEX};

use super::{proto::read_messages, util::sanitize_text, COLORED_USERNAMES, DATE_REGEX, config::Context, proto::send_message};


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
}


fn on_command(ctx: Arc<Context>, command: &str) -> Result<(), Box<dyn Error>> {
    let command = command.trim_start_matches("/");
    let (command, args) = command.split_once(" ").unwrap_or((&command, ""));
    let args = args.split(" ").collect::<Vec<&str>>();

    if command == "clear" {
        send_message(&ctx.host, 
            &prepare_message(ctx.clone(), 
                &format!("\r\x1B[1A{}", " ".repeat(64)).repeat(ctx.max_messages)
                ))?;
    } else if command == "spam" {
        send_message(&ctx.host, 
            &prepare_message(ctx.clone(), 
                &format!("\r\x1B[1A{}{}", args.join(" "), " ".repeat(10)).repeat(ctx.max_messages)
                ))?;
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
        let mut before = ctx.messages.packet_size();
        let start = SystemTime::now();
        let message = format!("Checking ping... {:X}", random::<u16>());
        send_message(&ctx.host, &message)?;
        loop {
            let data = read_messages(&ctx.host, ctx.max_messages, before).ok().flatten();

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
        send_message(&ctx.host, &format!("Ping = {}ms", start.elapsed().unwrap().as_millis()))?;
    }

    Ok(())
}


pub fn print_console(ctx: Arc<Context>, messages: Vec<String>, input: &str) -> Result<(), Box<dyn Error>> {
    let (width, height) = terminal::size()?;
    let (width, height) = (width as usize, height as usize);

    let scroll = ctx.scroll.load(Ordering::SeqCst);
    let scroll = (1f64 - scroll as f64 / messages.len() as f64) * (height) as f64;
    let scroll = scroll as usize;

    let formatted_messages = if ctx.disable_formatting {
        messages
    } else {
        messages[messages.len()-height-1..].into_iter()
            .flat_map(|o| string_chunks(&o, width as usize - 1))
            .enumerate()
            .map(|(i, (s, l))| {
                format!("{}{}{}", 
                    s, 
                    " ".repeat(width - 1 - l), 
                    if i == scroll {
                        "#"
                    } else {
                        "|"
                    }
                )
            }).collect::<Vec<String>>()
    };

    let text = format!(
        "{}\r\n> {}", 
        formatted_messages.join("\r\n"),
        input
    );

    let mut out = stdout().lock();
    write!(out, "{}", text)?;
    out.flush()?;

    Ok(())
}


fn prepare_message(context: Arc<Context>, message: &str) -> String {
    format!("{}{}{}",
        if !context.disable_hiding_ip {
            "\r\x07"
        } else {
            ""
        },
        message,
        if !context.disable_hiding_ip && message.chars().count() < 39 { 
            " ".repeat(39-message.chars().count()) 
        } else { 
            String::new()
        }
    )
}


fn format_message(ctx: Arc<Context>, message: String) -> Option<String> {
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

    let prefix = if ctx.enable_ip_viewing {
        if let Some(ip) = ip {
            format!("{}{} [{}]", ip, " ".repeat(15-ip.len()), date)
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


fn replace_input(cursor: usize, len: usize, text: &str) {
    let spaces = if text.chars().count() < len {
        len-text.chars().count()
    } else {
        0
    };
    write!(stdout(), 
        "{}{}{}{}", 
        MoveLeft(1).to_string().repeat(cursor), 
        text,
        " ".repeat(spaces), 
        MoveLeft(1).to_string().repeat(spaces)
    ).unwrap();
    stdout().lock().flush().unwrap();
}

fn replace_input_left(cursor: usize, len: usize, text: &str, left: usize) {
    let spaces = if text.chars().count() < len {
        len-text.chars().count()
    } else {
        0
    };
    write!(stdout(), 
        "{}{}{}{}", 
        MoveLeft(1).to_string().repeat(cursor), 
        text,
        " ".repeat(spaces), 
        MoveLeft(1).to_string().repeat(len-left)
    ).unwrap();
    stdout().lock().flush().unwrap();
}


fn poll_events(ctx: Arc<Context>) -> Result<(), Box<dyn Error>> {
    let mut history: Vec<String> = vec![String::new()];
    let mut history_cursor: usize = 0;
    let mut cursor: usize = 0;

    let input = ctx.input.clone();
    let messages = ctx.messages.clone();

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
                            replace_input(cursor, message.chars().count(), "");
                            input.write().unwrap().clear();

                            cursor = 0;

                            history_cursor = history.len()-1;
                            history.push(String::new());

                            if message.starts_with("/") && !ctx.disable_commands {
                                on_command(ctx.clone(), &message)?;
                            } else {
                                let message = ctx.message_format
                                    .replace("{name}", &ctx.name)
                                    .replace("{text}", &message);
                                send_message(&ctx.host, &message)?;
                            }
                        } else {
                            print_console(
                                ctx.clone(),
                                messages.messages(), 
                                ""
                            )?;
                        }
                    }
                    KeyCode::Backspace => {
                        if cursor == 0 || !(0..=history[history_cursor].len()).contains(&(cursor)) {
                            continue 
                        }
                        let len = input.read().unwrap().chars().count();
                        history[history_cursor].remove(cursor-1);
                        *input.write().unwrap() = history[history_cursor].clone();
                        replace_input_left(cursor, len, &history[history_cursor], cursor-1);
                        cursor -= 1;
                    }
                    KeyCode::Delete => {
                        if cursor == 0 || !(0..history[history_cursor].len()).contains(&(cursor)) {
                            continue 
                        }
                        let len = input.read().unwrap().chars().count();
                        history[history_cursor].remove(cursor);
                        *input.write().unwrap() = history[history_cursor].clone();
                        replace_input_left(cursor, len, &history[history_cursor], cursor);
                    }
                    KeyCode::Esc => {
                        disable_raw_mode()?;
                        break;
                    }
                    KeyCode::Up | KeyCode::Down => {
                        history_cursor = if event.code == KeyCode::Up {
                            max(history_cursor, 1) - 1
                        } else {
                            min(history_cursor + 1, history.len() - 1)
                        };
                        let len = input.read().unwrap().chars().count();
                        *input.write().unwrap() = history[history_cursor].clone();
                        replace_input(cursor, len, &history[history_cursor]);
                        cursor = history[history_cursor].chars().count();
                    }
                    KeyCode::PageUp => {

                    }
                    KeyCode::PageDown => {

                    }
                    KeyCode::Left => {
                        if cursor > 0 {
                            cursor -= 1;
                            write!(stdout(), "{}", MoveLeft(1).to_string(), ).unwrap();
                            stdout().lock().flush().unwrap();
                        }
                    }
                    KeyCode::Right => {
                        if cursor < history[history_cursor].len() {
                            cursor += 1;
                            write!(stdout(), "{}", MoveRight(1).to_string(), ).unwrap();
                            stdout().lock().flush().unwrap();
                        }
                    }
                    KeyCode::Char(c) => {
                        if event.modifiers.contains(KeyModifiers::CONTROL) && "zxcZXCячсЯЧС".contains(c) {
                            disable_raw_mode().unwrap();
                            break;
                        }
                        history[history_cursor].insert(cursor, c);
                        input.write().unwrap().insert(cursor, c);
                        write!(stdout(), "{}{}", 
                            history[history_cursor][cursor..].to_string(), 
                            MoveLeft(1).to_string().repeat(history[history_cursor].len()-cursor-1)
                        ).unwrap();
                        stdout().lock().flush().unwrap();
                        cursor += 1;
                    }
                    _ => {}
                }
            },
            Event::Paste(data) => {
                input.write().unwrap().push_str(&data);
                write!(stdout(), "{}", &data).unwrap();
                stdout().lock().flush().unwrap();
            },
            Event::Resize(_, _) => {
                print_console(
                    ctx.clone(),
                    messages.messages(), 
                    &input.read().unwrap()
                )?;
            },
            Event::Mouse(data) => {
                match data.kind {
                    MouseEventKind::ScrollUp => {

                    },
                    MouseEventKind::ScrollDown => {

                    },
                    _ => {}
                }
            }
            _ => {}
        }
    }

    Ok(())
}

pub fn recv_tick(ctx: Arc<Context>) -> Result<(), Box<dyn Error>> {
    if let Ok(Some((messages, size))) = read_messages(&ctx.host, ctx.max_messages, ctx.messages.packet_size()) {
        let messages: Vec<String> = if ctx.disable_formatting {
            messages 
        } else {
            messages.into_iter().flat_map(|o| format_message(ctx.clone(), o)).collect()
        };
        ctx.messages.update(messages.clone(), size);
        print_console(ctx.clone(), messages, &ctx.input.read().unwrap())?;
    }
    thread::sleep(Duration::from_millis(ctx.update_time as u64));
    Ok(())
}

pub fn run_main_loop(ctx: Arc<Context>) {
    enable_raw_mode().unwrap();

    thread::spawn({
        let ctx = ctx.clone();

        move || {
            loop { 
                recv_tick(ctx.clone()).expect("Error printing console");
            }
        }
    });

    poll_events(ctx).expect("Error while polling events");
}