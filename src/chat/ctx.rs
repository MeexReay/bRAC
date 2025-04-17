use std::sync::{Arc, RwLock};

use rand::random;

use super::{config::{Args, Config}, gui::ask_string, ChatContext};

pub struct Context {
    pub chat: Arc<RwLock<Option<Arc<ChatContext>>>>,
    pub host: String, 
    pub name: String, 
    pub disable_formatting: bool, 
    pub disable_commands: bool, 
    pub disable_hiding_ip: bool,
    pub message_format: String,
    pub update_time: usize,
    pub max_messages: usize,
    pub enable_ip_viewing: bool,
    pub enable_auth: bool,
    pub enable_ssl: bool,
    pub enable_chunked: bool,
}

impl Context {
    pub fn new(config: &Config, args: &Args) -> Context {
        Context {
            chat: Arc::new(RwLock::new(None)),
            message_format: args.message_format.clone().unwrap_or(config.message_format.clone()), 
            host: args.host.clone().unwrap_or(config.host.clone()), 
            name: args.name.clone()
                .or(config.name.clone())
                .unwrap_or_else(|| ask_string(
                    "Name", 
                    format!("Anon#{:X}", random::<u16>())
                )), 
            disable_formatting: args.disable_formatting, 
            disable_commands: args.disable_commands, 
            disable_hiding_ip: args.disable_ip_hiding,
            update_time: config.update_time,
            max_messages: config.max_messages,
            enable_ip_viewing: args.enable_users_ip_viewing || config.enable_ip_viewing,
            enable_auth: args.enable_auth || config.enable_auth,
            enable_ssl: args.enable_ssl || config.enable_ssl,
            enable_chunked: args.enable_chunked || config.enable_chunked,
        }
    }

    pub fn chat(&self) -> Arc<ChatContext> {
        self.chat.read().unwrap().clone().unwrap()
    }
}