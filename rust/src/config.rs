pub const MIDI_CC_NUMBER: u8 = 60; // CC number for Composition Crossfader Phase
pub const MIDI_CHANNEL: u8 = 1; // MIDI channel (1-based)

pub const LEFT_BUTTON_CC: u8 = 87; // "Fade to Left" button on USB X-Session Anschluss 1
pub const RIGHT_BUTTON_CC: u8 = 15; // "Fade to Right" button on USB X-Session Anschluss 1
pub const CROSSFADER_CC: u8 = 10; // Master Crossfader on USB X-Session Anschluss 1

pub const DURATIONS: [u64; 7] = [1, 2, 10, 30, 60, 300, 600]; // seconds, 0 -> 127
pub const DEFAULT_DURATION_INDEX: usize = 2; // 10 seconds

/// Output MIDI port (a virtual port your visuals/DJ software listens on).
/// Override with the MIDI_OUTPUT_PORT env var — required on Windows, since there's
/// no built-in virtual MIDI bus there (see README for loopMIDI setup).
pub fn output_port_name() -> String {
    std::env::var("MIDI_OUTPUT_PORT").unwrap_or_else(|_| default_output_port().to_string())
}

/// Input MIDI port for the USB controller. Override with MIDI_INPUT_PORT — the
/// port name macOS/Windows assign to the same hardware differs between OSes.
pub fn input_port_name() -> String {
    std::env::var("MIDI_INPUT_PORT").unwrap_or_else(|_| default_input_port().to_string())
}

#[cfg(target_os = "macos")]
fn default_output_port() -> &'static str {
    "IAC-Treiber Bus 1"
}
#[cfg(not(target_os = "macos"))]
fn default_output_port() -> &'static str {
    ""
}

#[cfg(target_os = "macos")]
fn default_input_port() -> &'static str {
    "USB X-Session Anschluss 1"
}
#[cfg(not(target_os = "macos"))]
fn default_input_port() -> &'static str {
    ""
}
