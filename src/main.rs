use std::sync::Arc;

use bRAC::proto::{connect, read_messages, send_message};
use bRAC::chat::{config::{get_config_path, load_config, Args}, ctx::Context, run_main_loop};
use clap::Parser;


fn main() {
    let args = Args::parse();
    
    let config_path = get_config_path();

    if args.config_path {
        print!("{}", config_path.to_string_lossy());
        return;
    }

    let mut config = load_config(config_path);

    if args.read_messages {
        let mut stream = connect(&config.host, config.ssl_enabled, config.proxy.clone()).expect("Error reading message");

        print!("{}", read_messages(
                &mut stream, 
                config.max_messages, 
                0,
                !config.ssl_enabled,
                false
            )
            .ok().flatten()
            .expect("Error reading messages").0.join("\n")
        );
    }

    if let Some(message) = &args.send_message {
        let mut stream = connect(&config.host, config.ssl_enabled, config.proxy.clone()).expect("Error sending message");

        send_message(
            &mut stream, 
            message
        ).expect("Error sending message");
    }

    if args.send_message.is_some() || args.read_messages {
        return;
    }

    args.patch_config(&mut config);
    
    let ctx = Arc::new(Context::new(&config));

    run_main_loop(ctx.clone());
}
