pub const LEFT_BUTTON_CC: u8 = 87; // "Fade to Left" button on USB X-Session Anschluss 1
pub const RIGHT_BUTTON_CC: u8 = 15; // "Fade to Right" button on USB X-Session Anschluss 1
pub const CROSSFADER_CC: u8 = 10; // Master Crossfader on USB X-Session Anschluss 1

pub const DURATIONS: [u64; 7] = [1, 2, 10, 30, 60, 300, 600]; // seconds, 0.0 -> 1.0
pub const DEFAULT_DURATION_INDEX: usize = 2; // 10 seconds

/// Input MIDI port for the USB controller. Override with MIDI_INPUT_PORT - the
/// port name macOS/Windows assign to the same hardware differs between OSes.
pub fn input_port_name() -> String {
    std::env::var("MIDI_INPUT_PORT").unwrap_or_else(|_| default_input_port().to_string())
}

#[cfg(target_os = "macos")]
fn default_input_port() -> &'static str {
    "USB X-Session Anschluss 1"
}
#[cfg(not(target_os = "macos"))]
fn default_input_port() -> &'static str {
    ""
}

/// Resolume (or any OSC-listening visuals software) connection details.
/// Resolume's default incoming OSC port is 7000.
pub fn osc_host() -> String {
    std::env::var("OSC_HOST").unwrap_or_else(|_| "127.0.0.1".to_string())
}

pub fn osc_port() -> u16 {
    std::env::var("OSC_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(7000)
}

/// The OSC address to send the crossfade value to. Resolume generates the exact address
/// for a given composition/parameter in its OSC output panel - adjust to match yours.
/// Confirmed for the Composition Crossfader: "/composition/crossfader/phase", float 0.0-1.0.
pub fn osc_address() -> String {
    std::env::var("OSC_ADDRESS").unwrap_or_else(|_| "/composition/crossfader/phase".to_string())
}
