# Install

Requires the Rust toolchain ([rustup.rs](https://rustup.rs)). Then from this `rust/` folder:

```
$ cargo build --release
```

# Configuration

Unlike the Python version, MIDI port names are **not hardcoded** — they differ between macOS
and Windows (and even between machines), so they're read from environment variables with a
substring match (e.g. `MIDI_INPUT_PORT=X-Session` matches any port name containing that text).

- `MIDI_OUTPUT_PORT` — the virtual MIDI port your visuals/DJ software listens on.
  - macOS default: `IAC-Treiber Bus 1` (the built-in IAC Driver bus).
  - Windows: no built-in virtual MIDI bus exists by default, but Windows 11 now ships
    **Windows MIDI Services**, which can create one natively — no third-party tool needed:
    1. Install the SDK/tools: `winget install --id Microsoft.MIDI.SDK`
    2. Open `MIDI Settings` (Start menu, or
       `C:\Program Files\Windows MIDI Services\Tools\Settings\MidiSettings.exe`) and create a
       **permanent loopback endpoint pair** (e.g. named `Crossfader A` / `Crossfader B`). Under
       the hood these are MIDI 2.0 endpoints, but they automatically bridge down to two MIDI
       1.0 in/out port pairs, which `mido`/`midir` (and old-API apps generally) see fine.
    3. Set `MIDI_OUTPUT_PORT` to one half of the pair (e.g. `Crossfader A`), and point your
       visuals/DJ software's MIDI input at the *other* half (`Crossfader B`).

    If you'd rather not use the (currently Release-Candidate/preview) native stack,
    [loopMIDI](https://www.tobias-erichsen.de/software/loopmidi.html) is the established
    third-party alternative — just be aware it can conflict with Windows MIDI Services if both
    are active at once.
- `MIDI_INPUT_PORT` — the USB controller's input port name.
  - macOS default: `USB X-Session Anschluss 1`.
  - Windows: the X-Session Pro is class-compliant — no driver install needed. It enumerates as
    `USB X-Session` (confirmed on Windows 11 24H2+/build 26200).
  - In general, if a name doesn't match, enumerate available ports by running the binary once
    with no env var set — it prints all available ports it couldn't match.

If a port can't be found, the binary prints the list of available ports it did see, to make
picking the right name easier.

# Usage

macOS:
```
$ MIDI_OUTPUT_PORT="IAC-Treiber Bus 1" MIDI_INPUT_PORT="USB X-Session Anschluss 1" ./target/release/midi-auto-crossfader
```

Windows (with a Windows MIDI Services loopback pair named `Crossfader A` / `Crossfader B`, as
set up above — verified working):
```
> $env:MIDI_OUTPUT_PORT="Crossfader A"; $env:MIDI_INPUT_PORT="USB X-Session"; .\target\release\midi-auto-crossfader.exe
```

- Default Duration: `10 seconds`
- Fade to Left: `Ctrl` + `Left Arrow`, or the USB controller's `⏴` button (lower left)
- Fade to Right: `Ctrl` + `Right Arrow`, or the USB controller's `⏵` button (lower left)
- Decrease Duration: `Ctrl` + `Down Arrow`
- Increase Duration: `Ctrl` + `Up Arrow`
- Quit: `Esc`
- Moving the USB controller's physical crossfader manually aborts any running auto-crossfade
  and adopts its current position as the new starting value.

Keyboard shortcuts are global (work while any app is focused) — on macOS this requires granting
Accessibility permissions to the terminal/binary; on Windows it may require running as the same
user session as the target app (no elevation needed for normal use).
