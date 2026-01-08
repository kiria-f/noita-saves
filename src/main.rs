mod commands;
mod config;
mod ui;
mod utils;

use console::style;

use crate::{
    commands::{CMD_MAP, CMD_SHORTCUTS},
    utils::SaveInfo,
};

fn main() {
    ui::welcome();
    loop {
        // Tell user we are already working at their request )
        ui::writeln("\nSaves:");
        ui::writeln("Loading...").update_later();

        // Get current progress and available saves
        let current_save_mb = SaveInfo::current();
        let saves_mb = SaveInfo::all();

        // Available actions for prompt
        let mut actions = if current_save_mb.is_some() {
            vec!["save", "load", "delete", "play", "quit"]
        } else {
            vec!["load", "delete", "play", "quit"]
        };

        // Print available saves
        if let Some(saves) = &saves_mb {
            if !saves.is_empty() {
                let i_width = saves.len().to_string().len();
                for (i, save) in saves.iter().enumerate() {
                    ui::writeln(&format!(
                        "{:i_width$} ❯ {}",
                        i + 1,
                        save.to_string(current_save_mb.as_ref())
                    ));
                }
            } else {
                ui::writelnln(&style("< Nothing >").dim().to_string());
            }
        } else {
            ui::error("Cannot load saves");
            actions = vec!["play", "quit"];
        }

        // Ask user for action
        let response = ui::ask(&ui::main_prompt(&actions));
        if response.is_empty() {
            continue;
        }
        ui::ln();

        // Parse user input
        let (cmd_name_or_alias, arg) = if let Some(splitted) = response.split_once(" ") {
            (splitted.0.trim(), Some(splitted.1.trim()))
        } else {
            (response.as_str(), None)
        };

        // Call command
        if let Some(cmd) = CMD_MAP.get(cmd_name_or_alias) {
            cmd(saves_mb.as_ref(), arg);
        } else if let Some(cmd_name) = CMD_SHORTCUTS.get(cmd_name_or_alias) {
            CMD_MAP.get(cmd_name).unwrap()(saves_mb.as_ref(), arg);
        } else {
            commands::not_found(cmd_name_or_alias);
        }

        // Exit if user wants to quit
        if ["q", "quit"].contains(&cmd_name_or_alias) {
            break;
        }
    }
}
