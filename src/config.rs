use std::{env, path::PathBuf, sync::LazyLock};

#[derive(Debug)]
pub struct Config {
    pub cache_file_name: String,
    pub game_dir_path: PathBuf,
    pub saves_dir_path: PathBuf,
    pub current_save_path: PathBuf,
}

static MAC_DEBUG: bool = true;

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    let common_location;
    if !MAC_DEBUG {
        common_location = PathBuf::from(env::var("APPDATA").unwrap().replace("Roaming", "LocalLow"));
    } else {
        common_location = PathBuf::from(env::var("HOME").unwrap()).join("tmp").join("noita-saves");
    }
    return Config {
        cache_file_name: String::from(".noita_saves_cache.json"),
        game_dir_path: common_location.join("Nolla_Games_Noita"),
        saves_dir_path: common_location.join("Nolla_Games_Noita_Saves"),
        current_save_path: common_location.join("Nolla_Games_Noita").join("save00"),
    };
});
