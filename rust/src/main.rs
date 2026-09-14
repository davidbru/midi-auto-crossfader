mod config;
mod crossfade;
mod midi;
mod osc;
mod ui;

use std::sync::Arc;
use std::time::{Duration, Instant};

use rdev::{listen, Event, EventType, Key};

use crossfade::{CrossfadeState, Direction};
use osc::OscOutput;
use ui::Ui;

fn main() {
    let osc_host = config::osc_host();
    let osc_port = config::osc_port();
    let osc_address = config::osc_address();
    let osc_output = match OscOutput::new(&osc_host, osc_port, &osc_address) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("[OSC] failed to set up output socket: {e}");
            std::process::exit(1);
        }
    };
    println!("[OSC] sending '{osc_address}' to {osc_host}:{osc_port}");

    // Resolved before the UI (and its pinned status lines) exists, so this can still use
    // plain println! - looking the port up doesn't need `state`, only connecting does.
    let input_name = config::input_port_name();
    let pending_input = if input_name.is_empty() {
        println!("[USB Controller] MIDI_INPUT_PORT not set, skipping USB controller input.");
        None
    } else {
        match midi::find_input_port(&input_name) {
            Ok(found) => Some(found),
            Err(e) => {
                println!("[USB Controller] {e}");
                None
            }
        }
    };

    // From here on, all output must go through `state.log()`/the UI - it owns the
    // terminal's pinned status lines, and a raw println! would corrupt their rendering.
    let ui = Ui::new(config::DURATIONS[config::DEFAULT_DURATION_INDEX], 0.5);
    let state = Arc::new(CrossfadeState::new(osc_output, ui));

    if let Some((midi_in, port)) = pending_input {
        let listener_state = Arc::clone(&state);
        let connection = midi_in.connect(
            &port,
            "crossfade-input",
            move |_stamp, message, _| handle_midi_message(message, &listener_state),
            (),
        );
        match connection {
            Ok(conn) => {
                state.log(format!("[USB Controller] connected (matched '{input_name}')"));
                // Leak the connection deliberately: it must outlive main() for the
                // callback to keep firing, and the process exits via std::process::exit.
                std::mem::forget(conn);
            }
            Err(e) => state.log(format!("[USB Controller] failed to connect: {e}")),
        }
    }

    state.log("Ready - Ctrl+< fade left, Ctrl+Y fade right, Ctrl+Q/W adjust duration, Esc quit.");

    let mut ctrl_pressed = false;
    let mut ctrl_released_at: Option<Instant> = None;
    if let Err(e) = listen(move |event| {
        handle_keyboard_event(event, &mut ctrl_pressed, &mut ctrl_released_at, &state)
    }) {
        eprintln!("Keyboard listener error: {e:?}");
        std::process::exit(1);
    }
}

fn handle_midi_message(message: &[u8], state: &Arc<CrossfadeState>) {
    if message.len() < 3 || message[0] & 0xF0 != 0xB0 {
        return; // not a Control Change message
    }
    let control = message[1];
    let value = message[2];

    if control == config::LEFT_BUTTON_CC {
        state.log("[USB Controller] fade-left button pressed");
        state.start(Direction::Left);
    } else if control == config::RIGHT_BUTTON_CC {
        state.log("[USB Controller] fade-right button pressed");
        state.start(Direction::Right);
    } else if control == config::CROSSFADER_CC {
        state.log("[USB Controller] crossfader moved manually: stopping automatic crossfade");
        state.set_value(value as f32 / 127.0);
        state.stop();
    }
}

// A Ctrl release reported a few milliseconds before the action key's press - which can happen
// on a fast key-chord even though the physical release came after - shouldn't drop the
// shortcut, so a release is still honored as "held" for a short grace window afterward.
const CTRL_RELEASE_GRACE: Duration = Duration::from_millis(75);

fn handle_keyboard_event(
    event: Event,
    ctrl_pressed: &mut bool,
    ctrl_released_at: &mut Option<Instant>,
    state: &Arc<CrossfadeState>,
) {
    match event.event_type {
        EventType::KeyPress(key) => {
            if matches!(key, Key::ControlLeft | Key::ControlRight) {
                *ctrl_pressed = true;
                return;
            }
            if key == Key::Escape {
                state.log("[Keyboard] Esc pressed: stopping and exiting");
                state.stop();
                state.finish_ui();
                std::process::exit(0);
            }

            let ctrl_held = *ctrl_pressed
                || ctrl_released_at.is_some_and(|t| t.elapsed() < CTRL_RELEASE_GRACE);

            // Matched on the physical key, not the produced character: holding Ctrl makes
            // Windows (and macOS) report a control character instead of the plain letter,
            // so character-based matching silently never fires for these combos. Arrow keys
            // are deliberately avoided here too - rdev's global hook only observes keystrokes,
            // it doesn't consume them, so the focused terminal still sees Ctrl+Up/Down and
            // scrolls its own buffer, which hides the pinned status lines.
            let is_action_key = matches!(key, Key::IntlBackslash | Key::KeyY | Key::KeyA | Key::KeyS);

            if !ctrl_held {
                if is_action_key {
                    state.log(format!(
                        "[Keyboard] {key:?} pressed without Ctrl held (name: {:?}) - ignored",
                        event.name
                    ));
                }
                return;
            }

            match key {
                // Physical key with "<" printed on it (German/ISO keyboards - the key next to
                // left Shift). Mirrors the original Mac setup's keyboard.
                Key::IntlBackslash => {
                    state.log("[Keyboard] Ctrl+< pressed: crossfade left");
                    state.start(Direction::Left);
                }
                Key::KeyY => {
                    state.log("[Keyboard] Ctrl+Y pressed: crossfade right");
                    state.start(Direction::Right);
                }
                // Not Ctrl+A/S: those are the classic terminal XON/XOFF flow-control
                // characters (Ctrl+S pauses output, Ctrl+Q resumes it) that some terminal
                // emulators (e.g. JediTerm, PhpStorm's integrated console) still honor,
                // which looked like the app hanging until the next keypress "resumed" it.
                Key::KeyQ => {
                    let seconds = state.adjust_duration(false);
                    state.log_with_duration(
                        format!("[Keyboard] Ctrl+Q pressed: decrease duration to {seconds} seconds"),
                        seconds,
                    );
                }
                Key::KeyW => {
                    let seconds = state.adjust_duration(true);
                    state.log_with_duration(
                        format!("[Keyboard] Ctrl+W pressed: increase duration to {seconds} seconds"),
                        seconds,
                    );
                }
                other => {
                    state.log(format!(
                        "[Keyboard] Ctrl+{other:?} pressed (name: {:?}) - no shortcut bound",
                        event.name
                    ));
                }
            }
        }
        EventType::KeyRelease(key) => {
            if matches!(key, Key::ControlLeft | Key::ControlRight) {
                *ctrl_pressed = false;
                *ctrl_released_at = Some(Instant::now());
            }
        }
        _ => {}
    }
}
