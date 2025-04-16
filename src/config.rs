use std::{str::FromStr, sync::{Arc, RwLock}};
#[allow(unused_imports)]
use std::{env, fs, path::{Path, PathBuf}, thread, time::Duration};
use colored::Colorize;
use rand::random;
use serde_yml;
use clap::Parser;

use crate::chat::ChatContext;

use super::util::get_input;

const MESSAGE_FORMAT: &str = "\u{B9AC}\u{3E70}<{name}> {text}";

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default = "default_message_format")]
    pub message_format: String,
    #[serde(default = "default_update_time")]
    pub update_time: usize,
    #[serde(default = "default_max_messages")]
    pub max_messages: usize,
    #[serde(default)]
    pub enable_ip_viewing: bool,
    #[serde(default)]
    pub disable_ip_hiding: bool,
    #[serde(default)]
    pub enable_auth: bool,
    #[serde(default)]
    pub enable_ssl: bool,
    #[serde(default)]
    pub enable_chunked: bool,
}

fn default_max_messages() -> usize { 200 }
fn default_update_time() -> usize { 50 }
fn default_host() -> String { "meex.lol:11234".to_string() }
fn default_message_format() -> String { MESSAGE_FORMAT.to_string() }

fn ask_usize(name: impl ToString, default: usize) -> usize {
    get_input(format!("{} (default: {}) {} ", name.to_string().bold(), default, ">".bold()).bright_yellow())
        .and_then(|o| o.parse().ok()).unwrap_or(default)
}

fn ask_string(name: impl ToString, default: impl ToString + Clone) -> String {
    ask_string_option(name, default.clone()).unwrap_or(default.to_string())
}

fn ask_string_option(name: impl ToString, default: impl ToString) -> Option<String> {
    let default = default.to_string();
    get_input(format!("{} (default: {}) {} ", name.to_string().bold(), default, ">".bold()).bright_yellow())
}

fn ask_bool(name: impl ToString, default: bool) -> bool {
    get_input(format!("{} (Y/N, default: {}) {} ", name.to_string().bold(), if default { "Y" } else { "N" }, ">".bold()).bright_yellow())
        .map(|o| o.to_lowercase() != "n")
        .unwrap_or(default)
}

pub fn configure(path: PathBuf) -> Config {
    println!("{}", "To configure the client, please answer a few questions. It won't take long.".yellow());
    println!("{}", "You can reconfigure client in any moment via `bRAC --configure`".yellow());
    println!("{}", format!("Config stores in path `{}`", path.to_string_lossy()).yellow());
    println!();

    let host = ask_string("Host", default_host());
    let name = ask_string_option("Name", "ask every time");
    let update_time = ask_usize("Update interval", default_update_time());
    let max_messages = ask_usize("Max messages", default_max_messages());
    let message_format = ask_string("Message format", default_message_format());
    let enable_ip_viewing = ask_bool("Enable users IP viewing?", true);
    let disable_ip_hiding = ask_bool("Enable your IP viewing?", false);
    let enable_auth = ask_bool("Enable auth-mode?", false);
    let enable_ssl = ask_bool("Enable SSL?", false);
    let enable_chunked = ask_bool("Enable chunked reading?", true);

    let config = Config {
        host,
        name,
        message_format,
        update_time,
        max_messages,
        enable_ip_viewing,
        disable_ip_hiding,
        enable_auth,
        enable_ssl,
        enable_chunked
    };

    let config_text = serde_yml::to_string(&config).expect("Config save error");
    fs::create_dir_all(&path.parent().expect("Config save error")).expect("Config save error");
    fs::write(&path, config_text).expect("Config save error");

    println!();
    println!("{}", "Config saved! You can reconfigure it in any moment via `bRAC --configure`".yellow());

    config
}

pub fn load_config(path: PathBuf) -> Config {
    if !fs::exists(&path).unwrap_or_default() {
        let config = configure(path.clone());
        thread::sleep(Duration::from_secs(4));
        config
    } else {
        let config = &fs::read_to_string(&path).expect("Config load error");
        serde_yml::from_str(config).expect("Config load error")
    }
}

pub fn get_config_path() -> PathBuf {
    let mut config_dir = PathBuf::from_str(".").unwrap();

    #[cfg(all(feature = "homedir", not(target_os = "windows")))]
    if let Some(dir) = {
        let home_dir = {
            use homedir::my_home;
            my_home().ok().flatten()
        };

        #[cfg(target_os = "linux")]
        let config_dir = {
            let home_dir = home_dir.map(|o| o.join("bRAC"));
            home_dir.map(|o| o.join(".config"))
        };

        #[cfg(target_os = "macos")]
        let config_dir = {
            let home_dir = home_dir.map(|o| o.join("bRAC"));
            home_dir.map(|o| o.join(".config"))
        };

        config_dir
    } {
        config_dir = dir;
    }

    #[cfg(target_os = "windows")]
    if let Some(dir) = {
        env::var("APPDATA")
            .ok()
            .and_then(|o| Some(PathBuf::from_str(&o).ok()?.join("bRAC")))
    } {
        config_dir = dir;
    }

    config_dir.join("config.yml")
}


// TODO: make args exactly match to config except --config-path and --configure, example: change from `disable_formatting: bool` to `formatting: Option<bool>``
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Print config path
    #[arg(short='p', long)]
    pub config_path: bool,

    /// Use specified host
    #[arg(short='H', long)]
    pub host: Option<String>,

    /// Use specified name
    #[arg(short='n', long)]
    pub name: Option<String>,

    /// Use specified message format
    #[arg(short='F', long)]
    pub message_format: Option<String>,

    /// Print unformatted messages from chat and exit
    #[arg(short='r', long)]
    pub read_messages: bool,

    /// Send unformatted message to chat and exit
    #[arg(short='s', long, value_name="MESSAGE")]
    pub send_message: Option<String>,

    /// Disable message formatting and sanitizing
    #[arg(short='f', long)]
    pub disable_formatting: bool,

    /// Disable slash commands
    #[arg(short='c', long)]
    pub disable_commands: bool,

    /// Disable ip hiding
    #[arg(short='i', long)]
    pub disable_ip_hiding: bool,

    /// Enable users IP viewing
    #[arg(short='v', long)]
    pub enable_users_ip_viewing: bool,

    /// Configure client
    #[arg(short='C', long)]
    pub configure: bool,

    /// Enable authentication
    #[arg(short='a', long)]
    pub enable_auth: bool,

    /// Enable SSL
    #[arg(short='S', long)]
    pub enable_ssl: bool,

    /// Enable chunked reading
    #[arg(short='u', long)]
    pub enable_chunked: bool,
}

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
            name: args.name.clone().or(config.name.clone()).unwrap_or_else(|| ask_string("Name", format!("Anon#{:X}", random::<u16>()))), 
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