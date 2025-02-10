#[allow(unused_imports)]
use std::{env, fs, path::{Path, PathBuf}, thread, time::Duration};
use homedir::my_home;
use serde_yml;

use super::get_input;

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
}

fn default_max_messages() -> usize { 100 }
fn default_update_time() -> usize { 50 }
fn default_host() -> String { "meex.lol:11234".to_string() }
fn default_message_format() -> String { MESSAGE_FORMAT.to_string() }

pub fn configure(path: PathBuf) -> Config {
    let host = get_input("Host (default: meex.lol:11234) > ").unwrap_or("meex.lol:11234".to_string());
    let name = get_input("Name (default: ask every time) > ");
    let update_time = get_input("Update interval (default: 50) > ").map(|o| o.parse().ok()).flatten().unwrap_or(50);
    let max_messages = get_input("Max messages (default: 100) > ").map(|o| o.parse().ok()).flatten().unwrap_or(100);
    let enable_ip_viewing = get_input("Enable users IP viewing? (Y/N, default: N) > ").map(|o| o.to_lowercase() != "n").unwrap_or(false);
    let disable_ip_hiding = get_input("Enable your IP viewing? (Y/N, default: N) > ").map(|o| o.to_lowercase() != "n").unwrap_or(false);

    let config = Config {
        host,
        name,
        message_format: MESSAGE_FORMAT.to_string(),
        update_time,
        max_messages,
        enable_ip_viewing,
        disable_ip_hiding
    };

    let config_text = serde_yml::to_string(&config).expect("Config save error");
    fs::create_dir_all(&path.parent().expect("Config save error")).expect("Config save error");
    fs::write(&path, config_text).expect("Config save error");
    println!("Config saved! You can edit it in the path got with `bRAC --config-path`");

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
    #[allow(unused_variables)]
    let config_path = Path::new("config.yml").to_path_buf();

    #[cfg(target_os = "linux")]
    let config_path = {
        let home_dir = my_home().ok().flatten().expect("Config find path error");
        home_dir.join(".config").join("bRAC").join("config.yml")
    };

    #[cfg(target_os = "macos")]
    let config_path = {
        let home_dir = my_home().ok().flatten().expect("Config find path error");
        home_dir.join(".config").join("bRAC").join("config.yml")
    };

    #[cfg(target_os = "windows")]
    let config_path = {
        let appdata = env::var("APPDATA").expect("Config find path error");
        Path::new(&appdata).join("bRAC").join("config.yml")
    };

    config_path
}