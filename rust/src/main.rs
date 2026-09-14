mod config;
mod crossfade;
mod midi;

use std::sync::Arc;

use rdev::{listen, Event, EventType, Key};

use crossfade::{CrossfadeState, Direction};

fn main() {
    let output_name = config::output_port_name();
    if output_name.is_empty() {
        eprintln!(
            "No output MIDI port configured for this OS. Set the MIDI_OUTPUT_PORT \
             environment variable (see rust/README.md)."
        );
        std::process::exit(1);
    }

    let output = match midi::open_output(&output_name) {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("[MIDI Output] {e}");
            std::process::exit(1);
        }
    };
    println!("[MIDI Output] connected (matched '{output_name}')");

    let state = Arc::new(CrossfadeState::new(output));

    let input_name = config::input_port_name();
    if input_name.is_empty() {
        println!("[USB Controller] MIDI_INPUT_PORT not set, skipping USB controller input.");
    } else {
        match midi::find_input_port(&input_name) {
            Ok((midi_in, port)) => {
                let listener_state = Arc::clone(&state);
                let connection = midi_in.connect(
                    &port,
                    "crossfade-input",
                    move |_stamp, message, _| handle_midi_message(message, &listener_state),
                    (),
                );
                match connection {
                    Ok(conn) => {
                        println!("[USB Controller] connected (matched '{input_name}')");
                        // Leak the connection deliberately: it must outlive main() for the
                        // callback to keep firing, and the process exits via std::process::exit.
                        std::mem::forget(conn);
                    }
                    Err(e) => eprintln!("[USB Controller] failed to connect: {e}"),
                }
            }
            Err(e) => println!("[USB Controller] {e}"),
        }
    }

    println!("Ready - Ctrl+Left/Right fade left/right, Ctrl+Up/Down adjust duration, Esc quit.");

    let mut ctrl_pressed = false;
    if let Err(e) = listen(move |event| handle_keyboard_event(event, &mut ctrl_pressed, &state)) {
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
        println!("[USB Controller] fade-left button pressed");
        state.start(Direction::Left);
    } else if control == config::RIGHT_BUTTON_CC {
        println!("[USB Controller] fade-right button pressed");
        state.start(Direction::Right);
    } else if control == config::CROSSFADER_CC {
        println!("[USB Controller] crossfader moved manually: stopping automatic crossfade");
        state.set_value(value);
        state.stop();
    }
}

fn handle_keyboard_event(event: Event, ctrl_pressed: &mut bool, state: &Arc<CrossfadeState>) {
    match event.event_type {
        EventType::KeyPress(key) => {
            if matches!(key, Key::ControlLeft | Key::ControlRight) {
                *ctrl_pressed = true;
                return;
            }
            if key == Key::Escape {
                println!("[Keyboard] Esc pressed: stopping and exiting");
                state.stop();
                std::process::exit(0);
            }
            if !*ctrl_pressed {
                return;
            }
            // Matched on the physical key, not the produced character: holding Ctrl makes
            // Windows (and macOS) report a control character instead of the plain letter,
            // so character-based matching silently never fires for these combos.
            match key {
                Key::UpArrow => {
                    println!("[Keyboard] Ctrl+Up pressed: increase duration");
                    state.adjust_duration(true);
                }
                Key::DownArrow => {
                    println!("[Keyboard] Ctrl+Down pressed: decrease duration");
                    state.adjust_duration(false);
                }
                Key::LeftArrow => {
                    println!("[Keyboard] Ctrl+Left pressed: crossfade left");
                    state.start(Direction::Left);
                }
                Key::RightArrow => {
                    println!("[Keyboard] Ctrl+Right pressed: crossfade right");
                    state.start(Direction::Right);
                }
                other => {
                    println!(
                        "[Keyboard] Ctrl+{other:?} pressed (name: {:?}) - no shortcut bound",
                        event.name
                    );
                }
            }
        }
        EventType::KeyRelease(key) => {
            if matches!(key, Key::ControlLeft | Key::ControlRight) {
                *ctrl_pressed = false;
            }
        }
        _ => {}
    }
}
