use console::style;

use crate::{
    config::CONFIG,
    ui,
    utils::{self, SaveInfo},
};
use std::{
    collections::{HashMap, HashSet},
    fs,
    sync::LazyLock,
};

fn cmd_test(_saves: Option<&Vec<SaveInfo>>, _arg: Option<&str>) {}

fn cmd_quit(_saves: Option<&Vec<SaveInfo>>, _arg: Option<&str>) {
    ui::writeln("Thx for using NoitaSaves! Have a nice day!");
    ui::ln();
}

pub fn cmd_save(saves: Option<&Vec<SaveInfo>>, arg: Option<&str>) {
    // Get save name
    let name;
    if let Some(given) = arg {
        name = given.to_string();
    } else {
        name = ui::ask("Save name");
        if name.is_empty() {
            ui::error("Save name cannot be empty");
            return;
        }
    };

    // Validate save name
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
        return;
    }

    // Create save
    if let Err(err) = utils::copy_dir_with_progress(
        &CONFIG.current_save_path,
        &CONFIG.saves_dir_path.join(name),
        false,
        true,
        Some("Saving"),
    ) {
        ui::error(&format!("Failed to save save: {}", err));
        return;
    }
}

pub fn cmd_load(saves_mb: Option<&Vec<SaveInfo>>, arg: Option<&str>) {
    // Unwrap saves_mb
    let saves;
    if let Some(saves_mb) = saves_mb {
        saves = saves_mb;
    } else {
        ui::error("Saves");
        return;
    }

    // Get index
    let index: usize;
    if let Some(present) = arg {
        if let Ok(parsed) = present.parse() {
            index = parsed;
        } else {
            ui::error(&format!("Invalid index: {present}"));
            return;
        }
    } else {
        let asked = ui::ask("Save index");
        if let Ok(parsed) = asked.parse() {
            index = parsed;
        } else {
            ui::error(&format!("Invalid index: {asked}",));
            return;
        }
    }

    // Get save
    let save: &SaveInfo;
    if let Some(s) = saves.get(index - 1) {
        save = s;
    } else {
        ui::error(&format!("No save found by index: {}", index));
        return;
    }

    // Load save
    if let Err(err) = fs::remove_dir_all(&CONFIG.current_save_path) {
        ui::error(&format!("Failed to delete current progress: {}", err));
        return;
    }
    if let Err(err) =
        utils::copy_dir_with_progress(&save.path, &CONFIG.current_save_path, true, false, Some("Loading save"))
    {
        ui::error(&format!("Failed to load save: {}", err));
        return;
    }
}

pub fn not_found(cmd: &str) {
    ui::error(&format!("No such command: \"{}\"", cmd));
}

pub static CMD_MAP: LazyLock<HashMap<&str, fn(Option<&Vec<SaveInfo>>, Option<&str>)>> = LazyLock::new(|| {
    HashMap::from([
        ("test", cmd_test as fn(Option<&Vec<SaveInfo>>, Option<&str>)),
        ("save", cmd_save),
        ("load", cmd_load),
        ("quit", cmd_quit),
    ])
});

pub static CMD_SHORTCUTS: LazyLock<HashMap<&str, &str>> =
    LazyLock::new(|| CMD_MAP.keys().map(|&cmd_name| (&cmd_name[..1], cmd_name)).collect());
