use std::sync::Arc;
use std::io::stdout;
use std::io::Write;

use colored::Colorize;

use super::{
    super::{
        config::Context, 
        proto::{connect, read_messages},
        util::get_input
    }, format_message, on_send_message
};

pub fn run_main_loop(ctx: Arc<Context>) {
    loop {
        match connect(&ctx.host, ctx.enable_ssl) { 
            Ok(mut stream) => {
                match read_messages(
                    &mut stream, 
                    ctx.max_messages, 
                    ctx.messages.packet_size(), 
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
                            ctx.messages.append_and_store(ctx.max_messages, messages.clone(), size);
                        } else {
                            ctx.messages.update(ctx.max_messages, messages.clone(), size);
                        }
                    }
                    Err(e) => {
                        let msg = format!("Read messages error: {}", e.to_string()).bright_red().to_string();
                        ctx.messages.append(ctx.max_messages, vec![msg]);
                    }
                    _ => {}
                }
            },
            Err(e) => {
                let msg = format!("Connect error: {}", e.to_string()).bright_red().to_string();
                ctx.messages.append(ctx.max_messages, vec![msg]);
            }
        }

        let messages = ctx.messages.messages();

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

        if let Some(message) = get_input("") {
            if let Err(e) = on_send_message(ctx.clone(), &message) {
                let msg = format!("Send message error: {}", e.to_string()).bright_red().to_string();
                ctx.messages.append(ctx.max_messages, vec![msg]);
            }
        }
    }
}