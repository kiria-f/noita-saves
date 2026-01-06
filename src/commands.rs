use crate::{ui, utils::Save};
use std::{collections::HashMap, sync::LazyLock};

fn cmd_test(_saves: &Vec<Save>, _arg: &str) {}

fn cmd_quit(_saves: &Vec<Save>, _arg: &str) {
    ui::writeln("Thx for using NoitaSaves! Have a nice day!");
    ui::ln();
}

pub fn not_found(cmd: &str) {
    ui::error(&format!("No such command: \"{}\"", cmd));
}

pub static CMD_MAP: LazyLock<HashMap<&str, fn(&Vec<Save>, &str)>> =
    LazyLock::new(|| HashMap::from([("test", cmd_test as fn(&Vec<Save>, &str)), ("quit", cmd_quit)]));

pub static CMD_SHORTCUTS: LazyLock<HashMap<&str, &str>> =
    LazyLock::new(|| CMD_MAP.keys().map(|&cmd_name| (&cmd_name[..1], cmd_name)).collect());
