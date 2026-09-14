use midir::{MidiIO, MidiInput, MidiInputPort, MidiOutput, MidiOutputConnection, Ignore};

pub fn open_output(name_contains: &str) -> Result<MidiOutputConnection, String> {
    let midi_out = MidiOutput::new("midi-auto-crossfader").map_err(|e| e.to_string())?;
    let ports = midi_out.ports();
    let matched = ports
        .iter()
        .find(|p| port_name_contains(&midi_out, p, name_contains))
        .cloned();

    match matched {
        Some(port) => {
            let name = midi_out.port_name(&port).unwrap_or_default();
            midi_out
                .connect(&port, "crossfade-output")
                .map_err(|e| format!("failed to connect to '{name}': {e}"))
        }
        None => Err(port_not_found_message(
            "output",
            name_contains,
            &port_names(&midi_out, &ports),
        )),
    }
}

/// Returns the opened MidiInput plus the matched port, so the caller can call
/// `.connect(...)` with its own callback (midir's connect() consumes MidiInput by value).
pub fn find_input_port(name_contains: &str) -> Result<(MidiInput, MidiInputPort), String> {
    let mut midi_in = MidiInput::new("midi-auto-crossfader").map_err(|e| e.to_string())?;
    midi_in.ignore(Ignore::None);
    let ports = midi_in.ports();
    let matched = ports
        .iter()
        .find(|p| port_name_contains(&midi_in, p, name_contains))
        .cloned();

    match matched {
        Some(port) => Ok((midi_in, port)),
        None => Err(port_not_found_message(
            "input",
            name_contains,
            &port_names(&midi_in, &ports),
        )),
    }
}

fn port_name_contains<T: MidiIO>(io: &T, port: &T::Port, needle: &str) -> bool {
    io.port_name(port)
        .map(|n| n.contains(needle))
        .unwrap_or(false)
}

fn port_names<T: MidiIO>(io: &T, ports: &[T::Port]) -> Vec<String> {
    ports.iter().filter_map(|p| io.port_name(p).ok()).collect()
}

fn port_not_found_message(kind: &str, wanted: &str, available: &[String]) -> String {
    if wanted.is_empty() {
        return format!(
            "No MIDI {kind} port name configured. Available {kind} ports: {available:?}"
        );
    }
    format!("MIDI {kind} port containing '{wanted}' not found. Available {kind} ports: {available:?}")
}
