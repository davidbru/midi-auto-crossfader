use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

const BAR_WIDTH: usize = 41;

/// Two pinned status lines (current duration, current crossfade position) that update
/// in place, with regular log messages scrolling above them. Cloning is cheap - all
/// clones share the same underlying terminal state, so this can be handed to every
/// thread that needs to log or update status (keyboard, MIDI input, crossfade loop).
#[derive(Clone)]
pub struct Ui {
    duration_line: ProgressBar,
    position_line: ProgressBar,
}

impl Ui {
    pub fn new(initial_duration_secs: u64, initial_value: f32) -> Self {
        let multi = MultiProgress::new();
        let style = ProgressStyle::with_template("{msg}").expect("static template is valid");

        let duration_line = multi.add(ProgressBar::new_spinner());
        duration_line.set_style(style.clone());

        let position_line = multi.add(ProgressBar::new_spinner());
        position_line.set_style(style);

        let ui = Self {
            duration_line,
            position_line,
        };
        ui.set_duration(initial_duration_secs);
        ui.set_position(initial_value, None);
        ui
    }

    /// Prints a log line above the two pinned status lines without corrupting them -
    /// use this instead of println!/eprintln! anywhere after `Ui::new` has been called.
    pub fn log(&self, msg: impl AsRef<str>) {
        self.position_line.println(msg.as_ref());
    }

    pub fn set_duration(&self, seconds: u64) {
        self.duration_line
            .set_message(format!("Duration: {seconds} seconds"));
    }

    /// `arrow` is the direction currently fading ('<' or '>'), or None when idle.
    pub fn set_position(&self, value: f32, arrow: Option<char>) {
        self.position_line.set_message(render_bar(value, arrow));
    }

    /// Leaves the two status lines in place and returns the terminal to normal
    /// scrolling - call before the process exits.
    pub fn finish(&self) {
        self.duration_line.finish();
        self.position_line.finish();
    }
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
