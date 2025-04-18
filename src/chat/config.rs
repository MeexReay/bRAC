use std::str::FromStr;
use std::{fs, path::PathBuf};
use serde_yml;
use serde_default::DefaultFromSerde;
use clap::Parser;

const MESSAGE_FORMAT: &str = "\u{B9AC}\u{3E70}<{name}> {text}";

fn default_true() -> bool { true }
pub fn default_max_messages() -> usize { 200 }
pub fn default_update_time() -> usize { 50 }
pub fn default_host() -> String { "meex.lol:11234".to_string() }
pub fn default_message_format() -> String { MESSAGE_FORMAT.to_string() }

#[derive(serde::Serialize, serde::Deserialize, DefaultFromSerde, Clone)]
pub struct Config {
    #[serde(default = "default_host")] pub host: String,
    #[serde(default)] pub name: Option<String>,
    #[serde(default = "default_message_format")] pub message_format: String,
    #[serde(default = "default_update_time")] pub update_time: usize,
    #[serde(default = "default_max_messages")] pub max_messages: usize,
    #[serde(default = "default_true")] pub hide_my_ip: bool,
    #[serde(default)] pub show_other_ip: bool,
    #[serde(default)] pub auth_enabled: bool,
    #[serde(default)] pub ssl_enabled: bool,
    #[serde(default = "default_true")] pub chunked_enabled: bool,
    #[serde(default = "default_true")] pub formatting_enabled: bool,
    #[serde(default = "default_true")] pub commands_enabled: bool,
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

pub fn load_config(path: PathBuf) -> Config {
    if !fs::exists(&path).unwrap_or_default() {
        let config = Config::default();
        let config_text = serde_yml::to_string(&config).expect("Config save error");
        fs::create_dir_all(&path.parent().expect("Config save error")).expect("Config save error");
        fs::write(&path, config_text).expect("Config save error");
        config
    } else {
        let config = &fs::read_to_string(&path).expect("Config load error");
        serde_yml::from_str(config).expect("Config load error")
    }
}

pub fn save_config(path: PathBuf, config: &Config) {
    let config_text = serde_yml::to_string(config).expect("Config save error");
    fs::create_dir_all(&path.parent().expect("Config save error")).expect("Config save error");
    fs::write(&path, config_text).expect("Config save error");
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Print config path
    #[arg(short='p', long)]
    pub config_path: bool,

    /// Print unformatted messages from chat and exit
    #[arg(short='r', long)]
    pub read_messages: bool,

    /// Send unformatted message to chat and exit
    #[arg(short='s', long, value_name="MESSAGE")]
    pub send_message: Option<String>,
    
    #[arg(short='H', long)] pub host: Option<String>,
    #[arg(short='n', long)] pub name: Option<String>,
    #[arg(long)] pub message_format: Option<String>,
    #[arg(long)] pub update_time: Option<usize>,
    #[arg(long)] pub max_messages: Option<usize>,
    #[arg(long)] pub hide_my_ip: Option<bool>,
    #[arg(long)] pub show_other_ip: Option<bool>,
    #[arg(long)] pub auth_enabled:Option <bool>,
    #[arg(long)] pub ssl_enabled: Option<bool>,
    #[arg(long)] pub chunked_enabled: Option<bool>,
    #[arg(long)] pub formatting_enabled: Option<bool>,
    #[arg(long)] pub commands_enabled: Option<bool>,
}

impl Args {
    pub fn patch_config(&self, config: &mut Config) {
        if let Some(v) = self.host.clone() { config.host = v }
        if let Some(v) = self.name.clone() { config.name = Some(v) }
        if let Some(v) = self.message_format.clone() { config.message_format = v }
        if let Some(v) = self.update_time.clone() { config.update_time = v }
        if let Some(v) = self.max_messages.clone() { config.max_messages = v }
        if let Some(v) = self.hide_my_ip.clone() { config.hide_my_ip = v }
        if let Some(v) = self.show_other_ip.clone() { config.show_other_ip = v }
        if let Some(v) = self.auth_enabled.clone() { config.auth_enabled = v }
        if let Some(v) = self.ssl_enabled.clone() { config.ssl_enabled = v }
        if let Some(v) = self.chunked_enabled.clone() { config.chunked_enabled = v }
        if let Some(v) = self.formatting_enabled.clone() { config.formatting_enabled = v }
        if let Some(v) = self.commands_enabled.clone() { config.commands_enabled = v }
    }
}