use std::sync::Arc;

use clap::Parser;
use colored::Color;
use config::{configure, get_config_path, load_config, Args, Context};
use proto::{connect, read_messages, send_message};
use regex::Regex;
use lazy_static::lazy_static;
use chat::run_main_loop;


lazy_static! {
    static ref DATE_REGEX: Regex = Regex::new(r"\[(.*?)\] (.*)").unwrap();
    static ref IP_REGEX: Regex = Regex::new(r"\{(.*?)\} (.*)").unwrap();
    static ref COLORED_USERNAMES: Vec<(Regex, Color)> = vec![
        (Regex::new(r"\u{B9AC}\u{3E70}<(.*?)> (.*)").unwrap(), Color::Green),             // bRAC
        (Regex::new(r"\u{2550}\u{2550}\u{2550}<(.*?)> (.*)").unwrap(), Color::BrightRed), // CRAB
        (Regex::new(r"\u{00B0}\u{0298}<(.*?)> (.*)").unwrap(), Color::Magenta),           // Mefidroniy
        (Regex::new(r"<(.*?)> (.*)").unwrap(), Color::Cyan),                              // clRAC
    ];
}


mod config;
mod chat;
mod proto;
mod util;


fn main() {
    let args = Args::parse();
    
    let config_path = get_config_path();

    if args.config_path {
        print!("{}", config_path.to_string_lossy());
        return;
    }

    if args.configure {
        configure(config_path);
        return;
    }

    let config = load_config(config_path);
    
    let ctx = Arc::new(Context::new(&config, &args));

    if args.read_messages {
        let mut stream = connect(&ctx.host, ctx.enable_ssl).expect("Error reading message");
        print!("{}", read_messages(
                &mut stream, 
                ctx.max_messages, 
                0,
                !ctx.enable_ssl
            )
            .ok().flatten()
            .expect("Error reading messages").0.join("\n")
        );
    }

    if let Some(message) = &args.send_message {
        send_message(&mut connect(&ctx.host, ctx.enable_ssl).expect("Error sending message"), message).expect("Error sending message");
    }

    if args.send_message.is_some() || args.read_messages {
        return;
    }

    run_main_loop(ctx.clone());
}
