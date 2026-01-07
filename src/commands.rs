use console::style;
use walkdir::WalkDir;

use crate::{
    config::CONFIG,
    ui,
    utils::{self, Save},
};
use std::{collections::HashMap, fs, sync::LazyLock};

fn cmd_test(_saves: &Vec<Save>, _arg: &str) {}

fn cmd_quit(_saves: &Vec<Save>, _arg: &str) {
    ui::writeln("Thx for using NoitaSaves! Have a nice day!");
    ui::ln();
}

pub fn cmd_load(saves: &Vec<Save>, arg: &str) {
    let index;
    if arg.is_empty() {
        index = saves.len();
    } else if let Ok(parsed) = arg.parse::<usize>() {
        index = parsed;
    } else {
        ui::error(&format!("Invalid index: {}", arg));
        return;
    }

    if let Some(save) = saves.get(index - 1) {
        ui::writeln(
            &[
                "Loading ",
                &style("[").dim().to_string(),
                &save.name,
                &style("]").dim().to_string(),
                "...",
            ]
            .join(""),
        )
        .update_later();
        if let Err(err) = fs::remove_dir_all(&CONFIG.current_save_path) {
            ui::error(&format!("Failed to delete old save: {}", err));
            return;
        }
        if let Err(err) = utils::copy_dir_with_progress(&save.path, &CONFIG.current_save_path, Some("Loading save")) {
            ui::error(&format!("Failed to load save: {}", err));
            return;
        }
    } else {
        ui::error(&format!("No save found by index: {}", index));
    }
}

pub fn not_found(cmd: &str) {
    ui::error(&format!("No such command: \"{}\"", cmd));
}

pub static CMD_MAP: LazyLock<HashMap<&str, fn(&Vec<Save>, &str)>> = LazyLock::new(|| {
    HashMap::from([
        ("test", cmd_test as fn(&Vec<Save>, &str)),
        ("quit", cmd_quit),
        ("load", cmd_load),
    ])
});

pub static CMD_SHORTCUTS: LazyLock<HashMap<&str, &str>> =
    LazyLock::new(|| CMD_MAP.keys().map(|&cmd_name| (&cmd_name[..1], cmd_name)).collect());
