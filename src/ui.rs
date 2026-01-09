use chrono::Duration;
use console::{Color, Term, style};
use regex::Regex;
use std::{
    io::{self, Write},
    sync::{LazyLock, Mutex},
    time::Instant,
};

use crate::config::DEBUG;

struct Memo {
    lines_to_update: Option<usize>,
}

pub struct PostHandler {
    lines_printed: usize,
}

impl PostHandler {
    fn from_output(output: &str) -> Self {
        PostHandler {
            lines_printed: output.split('\n').count(),
        }
    }

    pub fn update_later(&self) {
        MEMO.lock().unwrap().lines_to_update = Some(self.lines_printed);
    }
}

static TERM: LazyLock<Term> = LazyLock::new(|| Term::buffered_stdout());
static MEMO: LazyLock<Mutex<Memo>> = LazyLock::new(|| Mutex::new(Memo { lines_to_update: None }));

pub fn lnwrite(msg: &str) -> PostHandler {
    let mut memo = MEMO.lock().expect("Cannot access MEMO");
    if let Some(l2u) = memo.lines_to_update {
        TERM.clear_last_lines(l2u).ok();
        memo.lines_to_update = None;
    }

    TERM.write_line(msg).ok();
    TERM.flush().ok();

    return PostHandler::from_output(msg);
}

pub fn lnlnwrite(msg: &str) -> PostHandler {
    return lnwrite(&("\n".to_string() + msg));
}

pub fn lnlnwrite_highlighted(border_color: Color, msg: &str) -> PostHandler {
    let border = format!("{} ", style("┃").fg(border_color));
    let replacement = format!("\n{border}");
    let mut replaced = msg.replace("\n", &replacement);
    replaced.insert_str(0, &border);
    return lnlnwrite(&replaced);
}

pub fn error(msg: &str) {
    let mut buf = style("Error:\n").red().to_string();
    buf.push_str(msg);
    lnlnwrite_highlighted(Color::Red, &buf);
}

pub fn debug(msg: &str) {
    if DEBUG {
        let mut buf = style("Debug:\n").cyan().to_string();
        buf.push_str(msg);
        lnlnwrite_highlighted(Color::Cyan, &buf);
    }
}

pub fn dim_squares(text: String) -> String {
    Regex::new(r"\[(.)\]")
        .unwrap()
        .replace_all(&text, format!("{}{}{}", style("[").dim(), "$1", style("]").dim()))
        .to_string()
}

pub fn main_prompt(actions: &[&str]) -> String {
    dim_squares(
        actions
            .iter()
            .map(|&s| ["[", &s[..1].to_uppercase(), "]", &s[1..]].join(""))
            .collect::<Vec<_>>()
            .join(" | "),
    )
}

pub fn ask(prompt: &str) -> Option<String> {
    let mut memo = MEMO.lock().expect("Cannot access MEMO");
    if let Some(l2u) = memo.lines_to_update {
        TERM.clear_last_lines(l2u).ok();
        TERM.flush().ok();
        memo.lines_to_update = None;
    }

    let mut buf = String::new();
    print!("\n{}{}", prompt, style(" ❯ ").cyan());
    io::stdout().flush().ok();
    std::io::stdin().read_line(&mut buf).unwrap();
    let response = buf.trim().to_string();
    return Some(response).filter(|s| !s.is_empty());
}

pub struct ProgressBar {
    target: usize,
    title: Option<String>,
    status: usize,
    visible_status: u32,
    bar_width: u32,
    min_elapsed: Duration,
    redrawn: u32,
    time_point: Instant,
}

impl ProgressBar {
    pub fn new(target: usize, title: Option<&str>, framerate: u64) -> Self {
        let mut bar = Self {
            target,
            title: title.map(|t| t.to_string()),
            status: 0,
            visible_status: 0,
            bar_width: 20,
            min_elapsed: Duration::milliseconds(1000 / framerate as i64),
            redrawn: 0,
            time_point: Instant::now(),
        };
        bar.draw();
        return bar;
    }

    pub fn update(&mut self, status: usize) {
        self.status = status;
        let vs = ((self.status as f32 / self.target as f32) * self.bar_width as f32) as u32;
        if vs > self.visible_status && self.time_point.elapsed() >= self.min_elapsed.to_std().unwrap() {
            self.time_point = Instant::now();
            self.visible_status = vs;
            self.draw();
        }
    }

    fn draw(&mut self) {
        self.redrawn += 1;
        let filled = self.visible_status;
        let empty = self.bar_width - filled;

        if empty > 0 {
            let filled_bar = "█".repeat(filled as usize);
            let empty_bar = "░".repeat(empty as usize);

            self.visible_status = filled;
            lnlnwrite(&format!(
                "{}{}{}",
                self.title.as_ref().map_or(String::new(), |t| format!("{}: ", t)),
                filled_bar,
                empty_bar
            ))
            .update_later();
        } else {
            lnlnwrite(&format!(
                "{}{}",
                self.title.as_ref().map_or(String::new(), |t| format!("{}: ", t)),
                "Done!",
            ));
            debug(&format!(
                "Redrawn: {}\nElapsed: {:?}",
                self.redrawn,
                self.time_point.elapsed()
            ));
        }
    }
}

pub fn welcome() {
    let gh_link = style("https://github.com/kiria-f/noita-saves").cyan();

    lnlnwrite_highlighted(
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
