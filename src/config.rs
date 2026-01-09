use std::{env, path::PathBuf, sync::LazyLock};

use crate::ui::{debug, error};

#[derive(Debug)]
pub struct Config {
    pub cache_file_name: String,
    pub saves_dir_path: PathBuf,
    pub current_save_path: PathBuf,
}

pub static DEBUG: bool = false;
static DEBUG_LOCATION: bool = false;

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    let common_location;
    if !DEBUG_LOCATION {
        if let Ok(appdata) = env::var("APPDATA") {
            common_location = PathBuf::from(appdata.replace("Roaming", "LocalLow"));
        } else {
            error("Failed to get APPDATA environment variable");
            panic!();
        }
    } else {
        if let Ok(home) = env::var("USERPROFILE") {
            common_location = PathBuf::from(home).join("tmp").join("noita-saves");
        } else {
            error("Failed to get HOME environment variable");
            panic!();
        }
    }
    debug(&format!("Common location: {}", common_location.display()));
    return Config {
        cache_file_name: String::from(".noita_saves_cache.json"),
        saves_dir_path: common_location.join("Nolla_Games_Noita_Saves"),
        current_save_path: common_location.join("Nolla_Games_Noita").join("save00"),
    };
});
