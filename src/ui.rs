use bytesize::ByteSize;
use chrono::{DateTime, Local};
use console::{Color, Term, style};
use regex::Regex;
use std::{
    io::{self, Write},
    sync::{LazyLock, Mutex},
};

use crate::utils::{self, Save};

struct Memo {
    lines_to_update: Option<usize>,
}

pub struct PostHandler {
    lines_printed: usize,
}

impl PostHandler {
    fn from_output(&mut self, output: &str) {
        self.lines_printed = output.split('\n').count();
    }
    pub fn update_later(&self) {
        MEMO.lock().unwrap().lines_to_update = Some(self.lines_printed);
    }
}

static TERM: LazyLock<Term> = LazyLock::new(|| Term::stdout());
static MEMO: LazyLock<Mutex<Memo>> = LazyLock::new(|| Mutex::new(Memo { lines_to_update: None }));

fn write(msg: &str) {
    let mut memo = MEMO.lock().expect("Cannot access MEMO");
    if let Some(l2u) = memo.lines_to_update {
        TERM.clear_last_lines(l2u).ok();
    }
    memo.lines_to_update = None;
    print!("{}", msg);
    io::stdout().flush().ok();
}

pub fn writeln(msg: &str) -> PostHandler {
    let mut memo = MEMO.lock().expect("Cannot access MEMO");
    if let Some(l2u) = memo.lines_to_update {
        TERM.clear_last_lines(l2u).ok();
    }
    memo.lines_to_update = None;

    TERM.write_line(msg).ok();

    return PostHandler {
        lines_printed: msg.split('\n').count(),
    };
}

pub fn writelnln(msg: &str) -> PostHandler {
    let mut post_handler = writeln(msg);
    TERM.write_line("").ok();
    post_handler.lines_printed += 1;
    return post_handler;
}

pub fn ln() {
    writeln("");
}

pub fn writelnln_highlighted(border_color: Color, msg: &str) -> PostHandler {
    let border = format!("{} ", style("┃").fg(border_color));
    let replacement = format!("\n{border}");
    let mut replaced = msg.replace("\n", &replacement);
    replaced.insert_str(0, &border);
    return writelnln(&replaced);
}

pub fn error(msg: &str) {
    let mut buf = style("Error:\n").red().to_string();
    buf.push_str(msg);
    writelnln_highlighted(Color::Red, &buf);
}

pub fn welcome() {
    let gh_link = style("https://github.com/kiria-f/noita-saves").cyan();

    ln();
    writelnln_highlighted(
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
    fn construct_prompt(actions: &[&str]) -> String {
        Regex::new(r"\[(.)\]")
            .unwrap()
            .replace_all(
                &("\n".to_string()
                    + &actions
                        .iter()
                        .map(|&s| ["[", &s[..1].to_uppercase(), "]", &s[1..]].join(""))
                        .collect::<Vec<_>>()
                        .join(" | ")
                    + &style(" ❯ ").cyan().to_string()),
                format!("{}{}{}", style("[").dim(), "$1", style("]").dim()),
            )
            .to_string()
    }

    writeln("\nSaves:");
    writeln("Loading...").update_later();

    let current_save_mb = utils::get_current_save();
    if current_save_mb.is_none() {
        error("Cannot find current Noita progress");
    }
    let saves;
    if let Some(saves_mb) = utils::get_saves() {
        saves = saves_mb;
    } else {
        error("Cannot load saves");
        write(&construct_prompt(&["play", "quit"]));
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
                    &style(format!(
                        "{}  {}  {}",
                        &main_info,
                        &additional_info,
                        style("Current").dim()
                    ))
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

    write(&construct_prompt(&["save", "load", "play", "delete", "quit"]));
    return saves;
}
