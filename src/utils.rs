use std::{
    fs, io,
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::{config::CONFIG, ui::ProgressBar};
use bytesize::ByteSize;
use chrono::{DateTime, Local};
use console::style;
use walkdir::WalkDir;

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq)]
pub struct SaveStat {
    pub size: u64,
    pub count: usize,
}

impl SaveStat {
    pub fn read_cache(save_path: &Path) -> io::Result<SaveStat> {
        let content = fs::read_to_string(save_path.join(&CONFIG.cache_file_name))?;
        return Ok(serde_json::from_str::<SaveStat>(&content)?);
    }

    pub fn scan(save_path: &Path) -> SaveStat {
        SaveStat {
            size: WalkDir::new(save_path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .filter(|e| e.file_name().to_string_lossy() != CONFIG.cache_file_name)
                .filter_map(|e| e.metadata().ok())
                .map(|m| m.len())
                .sum(),
            count: WalkDir::new(save_path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .filter(|e| e.file_name().to_string_lossy() != CONFIG.cache_file_name)
                .count(),
        }
    }

    pub fn new(save_path: &Path, read_cache: bool) -> SaveStat {
        if read_cache {
            SaveStat::read_cache(save_path).unwrap_or_else(|_| SaveStat::scan(save_path))
        } else {
            SaveStat::scan(save_path)
        }
    }

    pub fn read_cache_or_scan(save_path: &Path) -> SaveStat {
        SaveStat::new(save_path, true)
    }

    pub fn write_cache(&self, save_path: &Path) -> io::Result<()> {
        let content = serde_json::to_string(self)?;
        fs::write(save_path.join(&CONFIG.cache_file_name), content)?;
        Ok(())
    }
}

pub struct SaveInfo {
    pub path: PathBuf,
    pub name: String,
    pub ctime: SystemTime,
    pub stat: SaveStat,
}

impl SaveInfo {
    pub fn current() -> Option<Self> {
        let meta = CONFIG.current_save_path.metadata().ok()?;
        return Some(SaveInfo {
            path: CONFIG.current_save_path.clone(),
            name: "".to_string(),
            ctime: meta.created().or_else(|_| meta.modified()).ok()?,
            stat: SaveStat::scan(&CONFIG.current_save_path),
        });
    }

    pub fn new(name: String) -> Option<Self> {
        let path = CONFIG.saves_dir_path.join(&name);
        if !path.exists() {
            return None;
        }
        let metadata = path.metadata().ok()?;
        let stat = SaveStat::read_cache_or_scan(&CONFIG.saves_dir_path.join(&name));
        return Some(SaveInfo {
            path,
            name,
            ctime: metadata.created().or_else(|_| metadata.modified()).ok()?,
            stat,
        });
    }

    pub fn all() -> Option<Vec<SaveInfo>> {
        let mut saves: Vec<SaveInfo> = CONFIG
            .saves_dir_path
            .read_dir()
            .ok()?
            .filter_map(|entry| SaveInfo::new(entry.ok()?.file_name().to_string_lossy().into_owned()))
            .collect();
        saves.sort_by_key(|save| save.ctime);
        return Some(saves);
    }

    pub fn is_current(&self, current_save: Option<&SaveInfo>) -> bool {
        matches!(&current_save, Some(current_save) if current_save.stat == self.stat)
    }

    pub fn to_string(&self, current_save: Option<&SaveInfo>) -> String {
        let additional_info = format!(
            "[{} | {}]",
            DateTime::<Local>::from(self.ctime).format("%b %-d %H:%M:%S"),
            ByteSize::b(self.stat.size)
        );

        if self.is_current(current_save) {
            return style(format!(
                "{}  {}  {}",
                &self.name,
                &additional_info,
                style("<Current>").dim()
            ))
            .green()
            .bold()
            .to_string();
        } else {
            return format!("{}  {}", &self.name, style(&additional_info).dim());
        }
    }
}

pub fn copy_dir_with_progress(
    src: &Path,
    dst: &Path,
    read_src_cache: bool,
    write_dst_cache: bool,
    progress_bar_title: Option<&str>,
) -> Result<(), io::Error> {
    let stat = SaveStat::new(src, read_src_cache);
    let mut bar = ProgressBar::new(stat.count, progress_bar_title, 60);
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
    if write_dst_cache {
        stat.write_cache(dst).ok();
    }
    return Ok(());
}
