use std::sync::Arc;

use bRAC::chat::{
    config::{get_config_path, load_config, Args},
    ctx::Context,
};
use bRAC::proto::{connect, read_messages, send_message};
use clap::Parser;

fn main() {
    #[cfg(feature = "winapi")]
    unsafe {
        winapi::um::wincon::FreeConsole()
    };

    let args = Args::parse();

    let config_path = get_config_path();

    if args.config_path {
        print!("{}", config_path.to_string_lossy());
        return;
    }

    let mut config = load_config(config_path).unwrap_or_default();

    args.patch_config(&mut config);

    if args.read_messages {
        let mut stream =
            connect(&config.host, config.proxy.clone()).expect("Error reading message");

        print!(
            "{}",
            read_messages(&mut stream, config.max_messages, 0, false)
                .ok()
                .flatten()
                .expect("Error reading messages")
                .0
                .join("\n")
        );
    }

    if let Some(message) = &args.send_message {
        let mut stream =
            connect(&config.host, config.proxy.clone()).expect("Error sending message");

        send_message(&mut stream, message).expect("Error sending message");
    }

    if args.send_message.is_some() || args.read_messages {
        return;
    }

    #[cfg(feature = "gtk")]
    {
        let ctx = Arc::new(Context::new(&config));

        use bRAC::chat::run_main_loop;
        run_main_loop(ctx.clone());
    }
}
