use clap::Parser;
use serde_default::DefaultFromSerde;
use serde_yml;
use std::{fs, path::PathBuf, error::Error};

const MESSAGE_FORMAT: &str = "\u{B9AC}\u{3E70}<{name}> {text}";

fn default_true() -> bool {
    true
}
pub fn default_max_messages() -> usize {
    200
}
pub fn default_update_time() -> usize {
    100
}
pub fn default_oof_update_time() -> usize {
    10000
}
pub fn default_konata_size() -> usize {
    100
}
pub fn default_host() -> String {
    "wracs://meex.lol".to_string()
}
pub fn default_message_format() -> String {
    MESSAGE_FORMAT.to_string()
}

#[derive(serde::Serialize, serde::Deserialize, DefaultFromSerde, Clone)]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default = "default_message_format")]
    pub message_format: String,
    #[serde(default = "default_update_time")]
    pub update_time: usize,
    #[serde(default = "default_oof_update_time")]
    pub oof_update_time: usize,
    #[serde(default = "default_max_messages")]
    pub max_messages: usize,
    #[serde(default = "default_konata_size")]
    pub konata_size: usize,
    #[serde(default)]
    pub remove_gui_shit: bool,
    #[serde(default = "default_true")]
    pub hide_my_ip: bool,
    #[serde(default)]
    pub show_other_ip: bool,
    #[serde(default = "default_true")]
    pub chunked_enabled: bool,
    #[serde(default = "default_true")]
    pub formatting_enabled: bool,
    #[serde(default = "default_true")]
    pub commands_enabled: bool,
    #[serde(default)]
    pub proxy: Option<String>,
    #[serde(default = "default_true")]
    pub notifications_enabled: bool,
    #[serde(default = "default_true")]
    pub new_ui_enabled: bool,
    #[serde(default)]
    pub debug_logs: bool,
    #[serde(default)]
    pub avatar: Option<String>,
}

#[cfg(target_os = "windows")]
pub fn get_config_path() -> PathBuf {
    use std::env;
    use std::str::FromStr;
    env::var("APPDATA")
        .ok()
        .and_then(|o| Some(PathBuf::from_str(&o).ok()?.join("bRAC")))
        .unwrap_or("bRAC/config.yml".into())
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn get_config_path() -> PathBuf {
    use homedir::my_home;
    my_home()
        .ok()
        .flatten()
        .map(|o| o.join(".config"))
        .map(|o| o.join("bRAC"))
        .unwrap_or("bRAC".into())
        .join("config.yml")
}

pub fn load_config(path: PathBuf) -> Result<Config, Box<dyn Error>> {
    if !fs::exists(&path).unwrap_or_default() {
        let config = Config::default();
        save_config(path, &config)?;
        Ok(config)
    } else {
        let config = &fs::read_to_string(&path)?;
        Ok(serde_yml::from_str(config)?)
    }
}

pub fn save_config(path: PathBuf, config: &Config) -> Result<(), Box<dyn Error>> {
    let config_text = serde_yml::to_string(config)?;
    fs::create_dir_all(&path.parent().ok_or::<Box<dyn Error>>("cant find parent".into())?)?;
    fs::write(&path, config_text)?;
    Ok(())
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Print config path
    #[arg(short = 'p', long)]
    pub config_path: bool,

    /// Print unformatted messages from chat and exit
    #[arg(short = 'r', long)]
    pub read_messages: bool,

    /// Send unformatted message to chat and exit
    #[arg(short = 's', long, value_name = "MESSAGE")]
    pub send_message: Option<String>,

    #[arg(short = 'H', long)]
    pub host: Option<String>,
    #[arg(short = 'n', long)]
    pub name: Option<String>,
    #[arg(long)]
    pub message_format: Option<String>,
    #[arg(long)]
    pub update_time: Option<usize>,
    #[arg(long)]
    pub oof_update_time: Option<usize>,
    #[arg(long)]
    pub max_messages: Option<usize>,
    #[arg(long)]
    pub konata_size: Option<usize>,
    #[arg(long)]
    pub hide_my_ip: Option<bool>,
    #[arg(long)]
    pub show_other_ip: Option<bool>,
    #[arg(long)]
    pub remove_gui_shit: Option<bool>,
    #[arg(long)]
    pub chunked_enabled: Option<bool>,
    #[arg(long)]
    pub formatting_enabled: Option<bool>,
    #[arg(long)]
    pub commands_enabled: Option<bool>,
    #[arg(long)]
    pub notifications_enabled: Option<bool>,
    #[arg(long)]
    pub new_ui_enabled: Option<bool>,
    #[arg(long)]
    pub proxy: Option<String>,
    #[arg(long)]
    pub avatar: Option<String>,
    #[arg(long)]
    pub debug_logs: bool,
}

impl Args {
    pub fn patch_config(&self, config: &mut Config) {
        if let Some(v) = self.host.clone() {
            config.host = v
        }
        if let Some(v) = self.name.clone() {
            config.name = Some(v)
        }
        if let Some(v) = self.proxy.clone() {
            config.proxy = Some(v)
        }
        if let Some(v) = self.message_format.clone() {
            config.message_format = v
        }
        if let Some(v) = self.update_time {
            config.update_time = v
        }
        if let Some(v) = self.oof_update_time {
            config.oof_update_time = v
        }
        if let Some(v) = self.max_messages {
            config.max_messages = v
        }
        if let Some(v) = self.konata_size {
            config.konata_size = v
        }
        if let Some(v) = self.hide_my_ip {
            config.hide_my_ip = v
        }
        if let Some(v) = self.show_other_ip {
            config.show_other_ip = v
        }
        if let Some(v) = self.remove_gui_shit {
            config.remove_gui_shit = v
        }
        if let Some(v) = self.chunked_enabled {
            config.chunked_enabled = v
        }
        if let Some(v) = self.formatting_enabled {
            config.formatting_enabled = v
        }
        if let Some(v) = self.commands_enabled {
            config.commands_enabled = v
        }
        if let Some(v) = self.notifications_enabled {
            config.notifications_enabled = v
        }
        if let Some(v) = self.new_ui_enabled {
            config.new_ui_enabled = v
        }
        if let Some(v) = self.avatar.clone() {
            config.avatar = Some(v)
        }
        if self.debug_logs {
            config.debug_logs = true
        }
    }
}
