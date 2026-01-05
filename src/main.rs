use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::{env, fs};
use walkdir::WalkDir;

#[derive(Debug)]
struct Config {
    cache_file_name: String,
    game_dir_path: PathBuf,
    saves_dir_path: PathBuf,
    current_save_path: PathBuf,
}

impl Config {
    fn new() -> Self {
        let appdata = PathBuf::from(env::var("APPDATA").unwrap().replace("Roaming", "LocalLow"));

        return Config {
            cache_file_name: String::from(".noita_saves_cache.json"),
            game_dir_path: appdata.join("Nolla_Games_Noita"),
            saves_dir_path: appdata.join("Nolla_Games_Noita_Saves"),
            current_save_path: appdata.join("Nolla_Games_Noita").join("save00"),
        };
    }
}

static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct SaveStat {
    size: u64,
    count: usize,
}

fn calc_dir_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

fn calc_dir_content(path: &Path) -> usize {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .count()
}

fn get_save_stat(save_name: &str) -> SaveStat {
    let config = CONFIG.get().unwrap();
    let save_path = config.saves_dir_path.join(save_name);
    let cache_path = save_path.join(&config.cache_file_name);

    if let Ok(content) = fs::read_to_string(&cache_path) {
        if let Ok(save_stat) = serde_json::from_str(&content) {
            return save_stat;
        }
    }

    let save_stat = SaveStat {
        size: calc_dir_size(&save_path),
        count: calc_dir_content(&save_path),
    };

    if let Ok(serialized) = serde_json::to_string(&save_stat) {
        fs::write(&cache_path, serialized).ok();
    }

    return save_stat;
}

fn main() {
    CONFIG.set(Config::new()).unwrap();
    let t = get_save_stat("m1");
    println!("{:?}", t);
}
