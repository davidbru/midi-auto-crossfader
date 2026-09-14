use std::io::{self, Write};
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Duration;

const BAR_WIDTH: usize = 41;

enum UiCommand {
    Log(String),
    LogWithDuration(String, u64),
    SetPosition(f32, Option<char>),
    Finish(Sender<()>),
}

/// Two pinned status lines (current duration, current crossfade position) that update
/// in place, with regular log messages scrolling above them via plain ANSI cursor moves.
///
/// All actual terminal writes happen on one dedicated thread owned by this type - this app
/// has at least three threads that want to update the UI concurrently (keyboard, MIDI input,
/// the crossfade loop), so every caller just drops a command on a channel and returns
/// immediately; only this one thread ever touches the terminal, so there's nothing for
/// concurrent writers to contend over. (An earlier version used the `indicatif` crate for
/// this, routed through the same single-thread channel - it still hung intermittently, so
/// this hand-rolled version replaced it: fewer moving parts to have a hidden lock in.)
/// Cloning `Ui` is cheap - clones just share the same channel sender.
#[derive(Clone)]
pub struct Ui {
    tx: Sender<UiCommand>,
}

impl Ui {
    pub fn new(initial_duration_secs: u64, initial_value: f32) -> Self {
        let (tx, rx) = mpsc::channel::<UiCommand>();

        thread::spawn(move || {
            let mut out = io::stdout().lock();
            let mut duration_text = format_duration(initial_duration_secs);
            let mut position_text = render_bar(initial_value, None);
            let mut drawn = false;

            draw(&mut out, &duration_text, &position_text, &mut drawn);

            for cmd in rx {
                match cmd {
                    UiCommand::Log(msg) => {
                        log_line(&mut out, &msg, &duration_text, &position_text, &mut drawn);
                    }
                    UiCommand::LogWithDuration(msg, seconds) => {
                        // Combined into one channel message (and so one atomic redraw) rather
                        // than a separate "update duration" + "log" pair sent back-to-back:
                        // two rapid-fire full redraws in a row confused at least one terminal
                        // (JetBrains' integrated console) into not repainting until later.
                        duration_text = format_duration(seconds);
                        log_line(&mut out, &msg, &duration_text, &position_text, &mut drawn);
                    }
                    UiCommand::SetPosition(value, arrow) => {
                        position_text = render_bar(value, arrow);
                        draw(&mut out, &duration_text, &position_text, &mut drawn);
                    }
                    UiCommand::Finish(done) => {
                        let _ = done.send(());
                        return;
                    }
                }
            }
        });

        Self { tx }
    }

    /// Prints a log line above the two pinned status lines - use this instead of
    /// println!/eprintln! anywhere after `Ui::new` has been called.
    pub fn log(&self, msg: impl AsRef<str>) {
        let _ = self.tx.send(UiCommand::Log(msg.as_ref().to_string()));
    }

    /// Updates the pinned duration line and prints a log line in one atomic redraw.
    pub fn log_with_duration(&self, msg: impl AsRef<str>, seconds: u64) {
        let _ = self
            .tx
            .send(UiCommand::LogWithDuration(msg.as_ref().to_string(), seconds));
    }

    /// `arrow` is the direction currently fading ('<' or '>'), or None when idle.
    pub fn set_position(&self, value: f32, arrow: Option<char>) {
        let _ = self.tx.send(UiCommand::SetPosition(value, arrow));
    }

    /// Blocks briefly for the UI thread to process any queued output before returning -
    /// call before the process exits, since std::process::exit right after would
    /// otherwise race it and could cut off the last few log lines.
    pub fn finish(&self) {
        let (done_tx, done_rx) = mpsc::channel();
        if self.tx.send(UiCommand::Finish(done_tx)).is_ok() {
            let _ = done_rx.recv_timeout(Duration::from_millis(500));
        }
    }
}

fn format_duration(seconds: u64) -> String {
    format!("Duration: {seconds} seconds")
}

/// Redraws the two pinned lines in place: move up over them (if already drawn) and
/// overwrite, clearing to end of line so a shorter new line doesn't leave stale characters.
fn draw(out: &mut impl Write, duration_text: &str, position_text: &str, drawn: &mut bool) {
    if *drawn {
        write!(out, "\x1b[2A").ok(); // cursor up 2 lines
    }
    write!(out, "\r\x1b[K{duration_text}\n\r\x1b[K{position_text}\n").ok();
    out.flush().ok();
    *drawn = true;
}

/// Prints a log line in place of the pinned lines, then redraws them below it - the net
/// effect is the log line scrolls up normally and the status lines stay pinned underneath.
fn log_line(out: &mut impl Write, msg: &str, duration_text: &str, position_text: &str, drawn: &mut bool) {
    if *drawn {
        write!(out, "\x1b[2A").ok();
    }
    write!(out, "\r\x1b[K{msg}\n").ok();
    write!(out, "\r\x1b[K{duration_text}\n\r\x1b[K{position_text}\n").ok();
    out.flush().ok();
    *drawn = true;
}

fn render_bar(value: f32, arrow: Option<char>) -> String {
    let mut chars = vec!['-'; BAR_WIDTH];
    let pos = (value.clamp(0.0, 1.0) * (BAR_WIDTH - 1) as f32).round() as usize;
    let pos = pos.min(BAR_WIDTH - 1);
    chars[pos] = '*';

    if let Some(arrow) = arrow {
        // The arrowhead sits on the side opposite its own travel direction, touching the
        // star as if pushing it that way: '<' (moving left) goes to the right of '*' ("*<"),
        // '>' (moving right) goes to the left of '*' (">*") - never trailing behind the move.
        let preferred = if arrow == '<' { pos + 1 } else { pos.wrapping_sub(1) };
        let fallback = if arrow == '<' { pos.wrapping_sub(1) } else { pos + 1 };
        let arrow_pos = if preferred < BAR_WIDTH && preferred != pos {
            preferred
        } else {
            fallback
        };
        if arrow_pos < BAR_WIDTH && arrow_pos != pos {
            chars[arrow_pos] = arrow;
        }
    }

    chars.into_iter().collect()
}
