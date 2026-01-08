use console::style;

use crate::{
    config::CONFIG,
    ui::{self, lnlnwrite},
    utils::{self, SaveInfo},
};
use std::{
    collections::{HashMap, HashSet},
    fs,
    sync::LazyLock,
};

// Interactive functions-helpers

fn interactive_check_saves_mb(saves_mb: Option<&Vec<SaveInfo>>) -> Option<&Vec<SaveInfo>> {
    saves_mb.or_else(|| {
        ui::error("Saves are unavailable");
        return None;
    })
}

fn interactive_parse_index(str_index: &str) -> Option<usize> {
    str_index.parse().ok().or_else(|| {
        ui::error(&format!("Invalid index: {str_index}"));
        return None;
    })
}

fn interactive_get_index_or_last(saves: &Vec<SaveInfo>, arg: Option<&str>) -> Option<usize> {
    match arg {
        None => Some(saves.len()),
        Some(s) => interactive_parse_index(s),
    }
}

fn interactive_validate_save_index(saves: &Vec<SaveInfo>, index: usize) -> Option<()> {
    if index == 0 {
        ui::error("Index must be greater than 0");
        return None;
    }
    if let Some(_) = saves.get(index - 1) {
        return Some(());
    } else {
        ui::error(&format!("No save found by index: {}", index));
        return None;
    }
}

fn interactive_get_save_by_index(saves: &Vec<SaveInfo>, index: usize) -> Option<&SaveInfo> {
    interactive_validate_save_index(saves, index)?;
    if let Some(save) = saves.get(index - 1) {
        return Some(save);
    } else {
        ui::error(&format!("No save found by index: {}", index));
        return None;
    }
}

fn interactive_parse_slice(saves: &Vec<SaveInfo>, str_slice: &str) -> Option<(usize, usize)> {
    if let Some(index) = str_slice.parse::<usize>().ok() {
        interactive_validate_save_index(saves, index)?;
        return Some((index, index));
    }
    if let Some((start, end)) = str_slice.split_once("..") {
        let start = start.parse::<usize>().or_else(|_| {
            if start.is_empty() {
                Result::Ok(1)
            } else {
                Result::Err(())
            }
        });
        let end = end.parse::<usize>().or_else(|_| {
            if end.is_empty() {
                Result::Ok(saves.len())
            } else {
                Result::Err(())
            }
        });
        if let (Ok(start), Ok(end)) = (start, end) {
            interactive_validate_save_index(saves, start)?;
            interactive_validate_save_index(saves, end)?;
            return Some((start, end));
        }
    }
    ui::error(&format!("Invalid interval: {str_slice}"));
    return None;
}

fn interactive_get_slice(saves: &Vec<SaveInfo>, arg: Option<&str>) -> Option<(usize, usize)> {
    interactive_parse_slice(
        saves,
        arg.map(|s| Some(s.to_string()))
            .unwrap_or_else(|| ui::ask("Save index or interval ([from]..[to])"))?
            .as_str(),
    )
}

fn interactive_get_saves_by_slice(saves: &Vec<SaveInfo>, slice: (usize, usize)) -> Option<&[SaveInfo]> {
    if let Some(saves) = saves.get(slice.0 - 1..=slice.1 - 1) {
        return Some(saves);
    } else {
        ui::error(&format!("No saves found by interval: {}..{}", slice.0, slice.1));
        return None;
    }
}

fn interactive_validate_save_name(saves: &Vec<SaveInfo>, name: &str) -> Option<()> {
    if name.is_empty() {
        ui::error("Save name cannot be empty");
        return None;
    }
    if name.len() > 69 {
        ui::error("Save name is too long");
        return None;
    }
    let forbidden_chars = name
        .chars()
        .filter(|&c| !(c.is_ascii_alphanumeric() || "( )=+-".contains(c)))
        .collect::<HashSet<char>>();
    if !forbidden_chars.is_empty() {
        ui::error(&format!(
            "Save name contains forbidden characters: {}{}{}",
            style("[").dim(),
            forbidden_chars.iter().collect::<String>(),
            style("]").dim()
        ));
        return None;
    }
    if saves.iter().any(|save| save.name == name) {
        ui::error(&format!("Save with this name already exists: {}", name));
        return None;
    }
    return Some(());
}

fn interactive_get_save_name(arg: Option<&str>) -> Option<String> {
    arg.map(|s| Some(s.to_string())).unwrap_or_else(|| ui::ask("Save name"))
}

// Commands

fn cmd_test(_saves: Option<&Vec<SaveInfo>>, _arg: Option<&str>) -> Option<()> {
    Some(())
}

fn cmd_save(saves_mb: Option<&Vec<SaveInfo>>, arg: Option<&str>) -> Option<()> {
    let saves = interactive_check_saves_mb(saves_mb)?;
    let save_name = interactive_get_save_name(arg)?;
    interactive_validate_save_name(saves, &save_name)?;

    if let Err(err) = utils::copy_dir_with_progress(
        &CONFIG.current_save_path,
        &CONFIG.saves_dir_path.join(save_name),
        false,
        true,
        Some("Saving"),
    ) {
        ui::error(&format!("Failed to save save: {}", err));
        return None;
    }
    return Some(());
}

fn cmd_load(saves_mb: Option<&Vec<SaveInfo>>, arg: Option<&str>) -> Option<()> {
    let saves = interactive_check_saves_mb(saves_mb)?;
    let index = interactive_get_index_or_last(saves, arg)?;
    let save = interactive_get_save_by_index(saves, index)?;

    if let Err(err) = fs::remove_dir_all(&CONFIG.current_save_path) {
        ui::error(&format!("Failed to delete current progress: {}", err));
        return None;
    }
    if let Err(err) =
        utils::copy_dir_with_progress(&save.path, &CONFIG.current_save_path, true, false, Some("Loading save"))
    {
        ui::error(&format!("Failed to load save: {}", err));
        return None;
    }
    return Some(());
}

fn cmd_delete(saves_mb: Option<&Vec<SaveInfo>>, arg: Option<&str>) -> Option<()> {
    let saves = interactive_check_saves_mb(saves_mb)?;
    let index = interactive_get_slice(saves, arg)?;
    let saves = interactive_get_saves_by_slice(saves, index)?;

    // Delete save
    if let Err(err) =
        utils::delete_dirs_with_progress(&saves.iter().map(|s| s.path.as_path()).collect(), Some("Deleting save"))
    {
        ui::error(&format!("Failed to delete save: {}", err));
        return None;
    }
    return Some(());
}

fn cmd_play(_saves_mb: Option<&Vec<SaveInfo>>, _arg: Option<&str>) -> Option<()> {
    lnlnwrite("Launching Noita...").update_later();
    open::that("steam://rungameid/881100").ok()
}

fn cmd_quit(_saves: Option<&Vec<SaveInfo>>, _arg: Option<&str>) -> Option<()> {
    ui::lnlnwrite("Thx for using NoitaSaves! Have a nice day!\n");
    return Some(());
}

pub fn cmd_not_found(cmd: &str) {
    ui::error(&format!("No such command: \"{}\"", cmd));
}

pub static CMD_MAP: LazyLock<HashMap<&str, fn(Option<&Vec<SaveInfo>>, Option<&str>) -> Option<()>>> =
    LazyLock::new(|| {
        HashMap::from([
            (
                "test",
                cmd_test as fn(Option<&Vec<SaveInfo>>, Option<&str>) -> Option<()>,
            ),
            ("save", cmd_save),
            ("load", cmd_load),
            ("delete", cmd_delete),
            ("play", cmd_play),
            ("quit", cmd_quit),
        ])
    });

pub static CMD_SHORTCUTS: LazyLock<HashMap<&str, &str>> =
    LazyLock::new(|| CMD_MAP.keys().map(|&cmd_name| (&cmd_name[..1], cmd_name)).collect());
