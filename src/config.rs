use std::{fs, path::{Path, PathBuf}};
use homedir::my_home;
use serde_yml;

use super::get_input;

const MESSAGE_FORMAT: &str = "\u{B9AC}\u{3E70}<{name}> {text}";

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
    pub host: String,
    pub name: Option<String>,
    pub message_format: String,
    pub update_time: usize,
    pub max_messages: usize
}

pub fn load_config(path: PathBuf) -> Config {
    // println!("Config path: {}", path.to_string_lossy());
    // println!("Loading config...");

    let config = if !fs::exists(&path).unwrap_or_default() {
        let host = get_input("Host (default: meex.lol:11234) > ").unwrap_or("meex.lol:11234".to_string());
        let name = get_input("Name (default: ask every time) > ");
        let update_time = get_input("Update Interval (default: 50) > ").map(|o| o.parse().ok()).flatten().unwrap_or(50);
        let max_messages = get_input("Max Messages (default: 100) > ").map(|o| o.parse().ok()).flatten().unwrap_or(100);

        let config = Config {
            host,
            name,
            message_format: MESSAGE_FORMAT.to_string(),
            update_time,
            max_messages
        };
        let config_text = serde_yml::to_string(&config).expect("Config save error");
        fs::create_dir_all(&path.parent().expect("Config save error")).expect("Config save error");
        fs::write(&path, config_text).expect("Config save error");
        config
    } else {
        let config = &fs::read_to_string(&path).expect("Config load error");
        serde_yml::from_str(config).expect("Config load error")
    };

    // println!("Config loaded successfully!");

    config
}

pub fn get_config_path() -> PathBuf {
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