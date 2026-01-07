use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::{config::CONFIG, ui::ProgressBar};
use walkdir::WalkDir;

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
pub struct SaveStat {
    pub size: u64,
    pub count: usize,
}

pub struct Save {
    pub path: PathBuf,
    pub name: String,
    pub ctime: SystemTime,
    pub stat: SaveStat,
}

fn calc_dir_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.file_name().to_string_lossy() != CONFIG.cache_file_name)
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

fn calc_dir_content(path: &Path) -> usize {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.file_name().to_string_lossy() != CONFIG.cache_file_name)
        .count()
}

pub fn copy_dir_with_progress(src: &Path, dst: &Path, progress_bar_title: Option<&str>) -> Result<(), std::io::Error> {
    let mut bar = ProgressBar::new(WalkDir::new(src).into_iter().count(), progress_bar_title);
    let mut progress = 0;
    for entry in WalkDir::new(src) {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(src_path.strip_prefix(src).unwrap());

        if entry.file_type().is_dir() {
            fs::create_dir(&dst_path)?;
        } else {
            if entry.file_name().to_string_lossy() != CONFIG.cache_file_name {
                fs::copy(src_path, &dst_path)?;
            }
        }
        progress += 1;
        bar.update(progress);
    }
    return Ok(());
}

fn get_dir_stat(path: &Path, use_cache: bool) -> Option<SaveStat> {
    if !path.exists() {
        return None;
    }
    let cache_path = path.join(&CONFIG.cache_file_name);

    if use_cache
        && let Ok(content) = fs::read_to_string(&cache_path)
        && let Ok(save_stat) = serde_json::from_str(&content)
    {
        return Some(save_stat);
    }

    let save_stat = SaveStat {
        size: calc_dir_size(&path),
        count: calc_dir_content(&path),
    };

    if use_cache && let Ok(serialized) = serde_json::to_string(&save_stat) {
        fs::write(&cache_path, serialized).ok();
    }

    return Some(save_stat);
}

pub fn get_saves() -> Option<Vec<Save>> {
    let mut saves: Vec<Save> = CONFIG
        .saves_dir_path
        .read_dir()
        .ok()?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let file_name_os = entry.file_name();
            let name = file_name_os.to_string_lossy().into_owned();
            let metadata = entry.metadata().ok()?;
            let ctime = metadata.created().or_else(|_| metadata.modified()).ok()?;
            let stat = get_dir_stat(&CONFIG.saves_dir_path.join(&name), true)?;
            return Some(Save {
                path: entry.path(),
                name,
                ctime,
                stat,
            });
        })
        .collect();
    saves.sort_by_key(|save| save.ctime);
    return Some(saves);
}

pub fn get_current_save() -> Option<Save> {
    let meta = CONFIG.current_save_path.metadata().ok()?;
    return Some(Save {
        path: CONFIG.current_save_path.clone(),
        name: "".to_string(),
        ctime: meta.created().or_else(|_| meta.modified()).ok()?,
        stat: get_dir_stat(&CONFIG.current_save_path, false)?,
    });
}
