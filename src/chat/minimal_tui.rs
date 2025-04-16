use std::sync::{Arc, RwLock};
use std::io::stdout;
use std::io::Write;
use std::error::Error;

use colored::Colorize;

use super::{
    super::{
        config::Context, 
        proto::{connect, read_messages},
        util::get_input
    }, format_message, on_send_message, ChatStorage, set_chat
};

pub struct ChatContext {
    pub messages: Arc<ChatStorage>, 
    pub registered: Arc<RwLock<Option<String>>>
}

fn update_console(ctx: Arc<Context>) -> Result<(), Box<dyn Error>> {
    let messages = ctx.chat().messages.messages();

    let mut out = stdout().lock();
    write!(
        out,
        "{}\n{}\n{} ",
        "\n".repeat(ctx.max_messages - messages.len()),
        messages
            .into_iter()
            .map(|o| o.white().blink().to_string())
            .collect::<Vec<String>>()
            .join("\n"),
        ">".bright_yellow()
    );
    out.flush();

    Ok(())
}

pub fn print_message(ctx: Arc<Context>, message: String) -> Result<(), Box<dyn Error>> {
    ctx.chat().messages.append(ctx.max_messages, vec![message]);
    update_console(ctx.clone())
}

pub fn run_main_loop(ctx: Arc<Context>) {
    set_chat(ctx.clone(), ChatContext {
        messages: Arc::new(ChatStorage::new()), 
        registered: Arc::new(RwLock::new(None)),
    });

    loop {
        match connect(&ctx.host, ctx.enable_ssl) { 
            Ok(mut stream) => {
                match read_messages(
                    &mut stream, 
                    ctx.max_messages, 
                    ctx.chat().messages.packet_size(), 
                    !ctx.enable_ssl,
                    ctx.enable_chunked
                ) {
                    Ok(Some((messages, size))) => {
                        let messages: Vec<String> = if ctx.disable_formatting {
                            messages 
                        } else {
                            messages.into_iter().flat_map(|o| format_message(ctx.enable_ip_viewing, o)).collect()
                        };

                        if ctx.enable_chunked {
                            ctx.chat().messages.append_and_store(ctx.max_messages, messages.clone(), size);
                        } else {
                            ctx.chat().messages.update(ctx.max_messages, messages.clone(), size);
                        }
                    }
                    Err(e) => {
                        let msg = format!("Read messages error: {}", e.to_string()).bright_red().to_string();
                        ctx.chat().messages.append(ctx.max_messages, vec![msg]);
                    }
                    _ => {}
                }
            },
            Err(e) => {
                let msg = format!("Connect error: {}", e.to_string()).bright_red().to_string();
                ctx.chat().messages.append(ctx.max_messages, vec![msg]);
            }
        }

        let _ = update_console(ctx.clone());

        if let Some(message) = get_input("") {
            if let Err(e) = on_send_message(ctx.clone(), &message) {
                let msg = format!("Send message error: {}", e.to_string()).bright_red().to_string();
                ctx.chat().messages.append(ctx.max_messages, vec![msg]);
            }
        }
    }
}