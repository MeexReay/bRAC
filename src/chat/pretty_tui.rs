use crossterm::{
  cursor::{MoveLeft, MoveRight}, 
  event::{self, Event, KeyCode, KeyModifiers, MouseEventKind}, 
  execute, 
  terminal::{self, disable_raw_mode, enable_raw_mode}
};

use colored::Colorize;

use std::{
    cmp::{max, min},
    error::Error, io::{stdout, Write}, 
    sync::{atomic::Ordering, Arc}, 
    thread, 
    time::Duration
};

use super::{
  super::{
    config::Context, proto::{connect, read_messages}, util::{char_index_to_byte_index, string_chunks}
  }, format_message, on_send_message
};


pub fn print_console(ctx: Arc<Context>, messages: Vec<String>, input: &str) -> Result<(), Box<dyn Error>> {
    let (width, height) = terminal::size()?;
    let (width, height) = (width as usize, height as usize);

    let mut messages = messages
        .into_iter()
        .flat_map(|o| string_chunks(&o, width as usize - 1))
        .map(|o| (o.0.white().blink().to_string(), o.1))
        .collect::<Vec<(String, usize)>>();

    let messages_size = if messages.len() >= height {
        messages.len()-height
    } else {
        for _ in 0..height-messages.len() {
            messages.insert(0, (String::new(), 0));
        }
        0
    };

    let scroll = min(ctx.scroll.load(Ordering::SeqCst), messages_size);
    let scroll_f = ((1f64 - scroll as f64 / (messages_size+1) as f64) * (height-2) as f64).round() as usize+1;

    let messages = if height < messages.len() {
        if scroll < messages.len() - height {
            messages[
                messages.len()-height-scroll..
                messages.len()-scroll
            ].to_vec()
        } else {
            if scroll < messages.len() {
                messages[
                    0..
                    messages.len()-scroll
                ].to_vec()
            } else {
                vec![]
            }
        }
    } else {
        messages
    };

    let formatted_messages = if ctx.disable_formatting {
        messages
            .into_iter()
            .map(|(i, _)| i)
            .collect::<Vec<String>>()
    } else {
        messages
            .into_iter()
            .enumerate()
            .map(|(i, (s, l))| {
                format!("{}{}{}", 
                    s, 
                    " ".repeat(width - 1 - l), 
                    if i == scroll_f {
                        "▐".bright_yellow()
                    } else {
                        "▕".yellow()
                    }
                )
            })
            .collect::<Vec<String>>()
            
    };

    let text = format!(
        "{}\r\n{} {}", 
        formatted_messages.join("\r\n"),
        ">".bright_yellow(),
        input
    );

    let mut out = stdout().lock();
    write!(out, "{}", text)?;
    out.flush()?;

    Ok(())
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

                            history.push(String::new());
                            history_cursor = history.len()-1;

                            if let Err(e) = on_send_message(ctx.clone(), &message) {
                                let msg = format!("Send message error: {}", e.to_string()).bright_red().to_string();
                                ctx.messages.append(ctx.max_messages, vec![msg]);
                                print_console(ctx.clone(), ctx.messages.messages(), &ctx.input.read().unwrap())?;
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
                        let i = char_index_to_byte_index(&history[history_cursor], cursor-1);
                        history[history_cursor].remove(i);
                        *input.write().unwrap() = history[history_cursor].clone();
                        replace_input_left(cursor, len, &history[history_cursor], cursor-1);
                        cursor -= 1;
                    }
                    KeyCode::Delete => {
                        if cursor == 0 || !(0..history[history_cursor].len()).contains(&(cursor)) {
                            continue 
                        }
                        let len = input.read().unwrap().chars().count();
                        let i = char_index_to_byte_index(&history[history_cursor], cursor);
                        history[history_cursor].remove(i);
                        *input.write().unwrap() = history[history_cursor].clone();
                        replace_input_left(cursor, len, &history[history_cursor], cursor);
                    }
                    KeyCode::Esc => {
                        on_close();
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
                        let height = terminal::size().unwrap().1 as usize;
                        ctx.scroll.store(min(ctx.scroll.load(Ordering::SeqCst)+height, ctx.messages.messages().len()), Ordering::SeqCst);
                        print_console(
                            ctx.clone(),
                            messages.messages(), 
                            &input.read().unwrap()
                        )?;
                    }
                    KeyCode::PageDown => {
                        let height = terminal::size().unwrap().1 as usize;
                        ctx.scroll.store(max(ctx.scroll.load(Ordering::SeqCst), height)-height, Ordering::SeqCst);
                        print_console(
                            ctx.clone(),
                            messages.messages(), 
                            &input.read().unwrap()
                        )?;
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
                            on_close();
                            break;
                        }
                        let i = char_index_to_byte_index(&history[history_cursor], cursor);
                        history[history_cursor].insert(i, c);
                        input.write().unwrap().insert(i, c);
                        write!(stdout(), "{}{}", 
                            history[history_cursor][i..].to_string(), 
                            MoveLeft(1).to_string().repeat(history[history_cursor].chars().count()-cursor-1)
                        ).unwrap();
                        stdout().lock().flush().unwrap();
                        cursor += 1;
                    }
                    _ => {}
                }
            },
            Event::Paste(data) => {
                let i = char_index_to_byte_index(&history[history_cursor], cursor);
                history[history_cursor].insert_str(i, &data);
                input.write().unwrap().insert_str(i, &data);
                write!(stdout(), "{}{}", 
                    history[history_cursor][cursor..].to_string(), 
                    MoveLeft(1).to_string().repeat(history[history_cursor].len()-cursor-1)
                ).unwrap();
                stdout().lock().flush().unwrap();
                cursor += data.len();
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
                        ctx.scroll.store(min(ctx.scroll.load(Ordering::SeqCst)+3, ctx.messages.messages().len()), Ordering::SeqCst);
                        print_console(
                            ctx.clone(),
                            messages.messages(), 
                            &input.read().unwrap()
                        )?;
                    },
                    MouseEventKind::ScrollDown => {
                        ctx.scroll.store(max(ctx.scroll.load(Ordering::SeqCst), 3)-3, Ordering::SeqCst);
                        print_console(
                            ctx.clone(),
                            messages.messages(), 
                            &input.read().unwrap()
                        )?;
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
    match read_messages(
        &mut connect(&ctx.host, ctx.enable_ssl)?, 
        ctx.max_messages, 
        ctx.messages.packet_size(), 
        !ctx.enable_ssl,
        ctx.enable_chunked
    ) {
        Ok(Some((messages, size))) => {
            let messages: Vec<String> = if ctx.disable_formatting {
                messages 
            } else {
                messages.into_iter().flat_map(|o| format_message(ctx.clone(), o)).collect()
            };

            if ctx.enable_chunked {
                ctx.messages.append_and_store(ctx.max_messages, messages.clone(), size);
                print_console(ctx.clone(), ctx.messages.messages(), &ctx.input.read().unwrap())?;
            } else {
                ctx.messages.update(ctx.max_messages, messages.clone(), size);
                print_console(ctx.clone(), messages, &ctx.input.read().unwrap())?;
            }
        },
        Err(e) => {
            let msg = format!("Read messages error: {}", e.to_string()).bright_red().to_string();
            ctx.messages.append(ctx.max_messages, vec![msg]);
            print_console(ctx.clone(), ctx.messages.messages(), &ctx.input.read().unwrap())?;
        }
        _ => {}
    }
    thread::sleep(Duration::from_millis(ctx.update_time as u64));
    Ok(())
}

pub fn on_close() {
    disable_raw_mode().unwrap();
    execute!(stdout(), event::DisableMouseCapture).unwrap();
}

pub fn run_main_loop(ctx: Arc<Context>) {
    enable_raw_mode().unwrap();
    execute!(stdout(), event::EnableMouseCapture).unwrap();

    if let Err(e) = print_console(ctx.clone(), Vec::new(), &ctx.input.read().unwrap()) {
        let msg = format!("Print messages error: {}", e.to_string()).bright_red().to_string();
        ctx.messages.append(ctx.max_messages, vec![msg]);
        let _ = print_console(ctx.clone(), ctx.messages.messages(), &ctx.input.read().unwrap());
    }

    thread::spawn({
        let ctx = ctx.clone();

        move || {
            loop { 
                if let Err(e) = recv_tick(ctx.clone()) {
                    let msg = format!("Print messages error: {}", e.to_string()).bright_red().to_string();
                    ctx.messages.append(ctx.max_messages, vec![msg]);
                    let _ = print_console(ctx.clone(), ctx.messages.messages(), &ctx.input.read().unwrap());
                    thread::sleep(Duration::from_secs(1));
                }
            }
        }
    });

    if let Err(e) = poll_events(ctx.clone()) {
        let msg = format!("Poll events error: {}", e.to_string()).bright_red().to_string();
        ctx.messages.append(ctx.max_messages, vec![msg]);
        let _ = print_console(ctx.clone(), ctx.messages.messages(), &ctx.input.read().unwrap());
    }
}