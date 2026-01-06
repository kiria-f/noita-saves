mod commands;
mod config;
mod ui;
mod utils;

use crate::commands::{CMD_MAP, CMD_SHORTCUTS};

fn main() {
    ui::welcome();
    loop {
        let saves = ui::prompt();
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        if line.trim().is_empty() {
            continue;
        }
        ui::ln();
        let (cmd_name_or_alias, arg) = if let Some(splitted) = line.split_once(" ") {
            (splitted.0.trim(), splitted.1.trim())
        } else {
            (line.as_str().trim(), "")
        };

        if let Some(cmd) = CMD_MAP.get(cmd_name_or_alias) {
            cmd(&saves, arg);
        } else if let Some(cmd_name) = CMD_SHORTCUTS.get(cmd_name_or_alias) {
            CMD_MAP.get(cmd_name).unwrap()(&saves, arg);
        } else {
            commands::not_found(cmd_name_or_alias);
        }

        if ["q", "quit", "exit"].contains(&cmd_name_or_alias) {
            break;
        }
    }
}
