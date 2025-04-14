use std::sync::Arc;

use clap::Parser;
use bRAC::config::{configure, get_config_path, load_config, Args, Context};
use bRAC::proto::{connect, read_messages, send_message};
use bRAC::chat::run_main_loop;


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
                !ctx.enable_ssl,
                false
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
