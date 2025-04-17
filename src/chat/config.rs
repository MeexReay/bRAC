use std::str::FromStr;
use std::{fs, path::PathBuf, thread, time::Duration};
use serde_yml;
use clap::Parser;

use super::gui::{ask_bool, ask_string, ask_string_option, ask_usize, show_message};

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

pub fn configure(path: PathBuf) -> Config {
    show_message("Client setup", format!("To configure the client, please answer a few questions. It won't take long.
You can reconfigure client in any moment via `bRAC --configure`
Config stores in path `{}`", path.to_string_lossy()));

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

    show_message("Config saved!", "You can reconfigure it in any moment via `bRAC --configure`");

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

    #[cfg(not(target_os = "windows"))]
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