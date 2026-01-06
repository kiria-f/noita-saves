use bytesize::ByteSize;
use chrono::{DateTime, Local};
use console::{Color, Term, style};
use regex::Regex;
use std::{
    io::{self, Write},
    sync::LazyLock,
};

use crate::utils::{self, Save};

static TERM: LazyLock<Term> = LazyLock::new(|| Term::stdout());

pub fn write(msg: &str) {
    print!("{}", msg);
    io::stdout().flush().ok();
}

pub fn writeln(msg: &str) {
    println!("{}", msg);
}

pub fn ln() {
    println!();
}

pub fn writeln_highlighted(border_color: Color, msg: &str) {
    let border = format!("{} ", style("┃").fg(border_color));
    let replacement = format!("\n{border}");
    write(&border);
    writeln(&msg.replace("\n", &replacement));
}

pub fn error(msg: &str) {
    writeln(&style("┃ Error:").red().to_string());
    writeln(&format!("{} {}", style("┃").red(), msg));
}

pub fn welcome() {
    let quote_bar = style("┃").yellow();
    let gh_link = style("https://github.com/kiria-f/noita-saves").cyan();

    ln();
    writeln_highlighted(
        Color::Green,
        &[
            &style("Welcome to NoitaSaves!").bold().green().to_string(),
            "",
            &format!("{}", style("To make a save, you should first quit the game").bold()),
            &format!("{}", style("You also need to close Noita before loading a save").bold()),
            "Turn off Steam sync in the game settings (if it's enabled)",
            "  Otherwise, do not load a save during Steam sync, it may corrupt the current game state",
            "  If the selected save has not loaded, just load it one more time",
            "  (It may happen due to steam sync)",
            "You can also create a shortcut for NoitaSaves on your start menu or desktop",
            &format!("(Check GitHub repo for more info: {gh_link})"),
        ]
        .join("\n"),
    );
}

pub fn prompt() -> Vec<Save> {
    writeln("\n\nSaves:");
    write("Loading...");

    let current_save_mb = utils::get_current_save();
    if current_save_mb.is_none() {
        error("Cannot find current Noita progress");
    }
    let saves;
    if let Some(saves_mb) = utils::get_saves() {
        saves = saves_mb;
        update("");
    } else {
        update("");
        error("Cannot load saves");
        return Vec::new();
    }

    if saves.len() > 0 {
        let i_width = saves.len().to_string().len();
        for i in 0..saves.len() {
            let save = &saves[i];
            let save_is_current = matches!(&current_save_mb, Some(current_save) if current_save.stat == save.stat);

            let main_info = format!("#{:i_width$} ❯ {}", i + 1, save.name);
            let additional_info = format!(
                "[{} | {}]",
                DateTime::<Local>::from(save.ctime).format("%b %-d %H:%M:%S"),
                ByteSize::b(save.stat.size)
            );

            if save_is_current {
                writeln(
                    &style(format!("{}  {}", &main_info, &additional_info))
                        .green()
                        .bold()
                        .to_string(),
                );
            } else {
                writeln(&format!("{}  {}", &main_info, style(&additional_info).dim()));
            }
        }
    } else {
        writeln("< Nothing >");
    }

    let re = Regex::new(r"\[(.)\]").unwrap();
    let boring_prompt = format!("\n[S]ave | [L]oad | [P]lay | [D]elete | [Q]uit {} ", style("❯").cyan());
    let replacement = format!("{}{}{}", style("[").dim(), "$1", style("]").dim());
    write(re.replace_all(&boring_prompt, &replacement).as_ref());

    return saves;
}

pub fn update(msg: &str) {
    TERM.clear_line().ok();
    write(msg);
}
