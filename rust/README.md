# Install

Requires the Rust toolchain ([rustup.rs](https://rustup.rs)). Then from this `rust/` folder:

```
$ cargo build --release
```

# Configuration

Output goes over **OSC** (not MIDI) — this avoids the whole virtual-MIDI-port dependency
(loopMIDI / Windows MIDI Services) on the output side entirely, and gives smooth float
resolution instead of MIDI's 128-step CC values. It's just a UDP packet to whatever OSC-listening
visuals software you're using (e.g. Resolume Arena/Avenue).

- `OSC_HOST` — destination host. Default: `127.0.0.1` (same machine).
- `OSC_PORT` — destination port. Default: `7000` (Resolume's default incoming OSC port).
- `OSC_ADDRESS` — the OSC address to send the crossfade value (a float, `0.0`-`1.0`) to.
  Default: `/composition/crossfader/phase` (confirmed against Resolume's own OSC input info
  panel — click the Composition Crossfader control to see a parameter's exact address).

In Resolume, enable OSC input (`Preferences` → `OSC`, port `7000` by default) and set up the
Composition Crossfader (assign layers to its two groups). Resolume requires an explicit
OSC-learn step before a parameter responds to incoming messages, the same as MIDI-learn:
right-click the crossfader, enable OSC learn, and trigger a brief fade from this app so it can
bind - just receiving/logging OSC traffic in the Preferences monitor isn't enough on its own.

Input (the USB controller) is still MIDI, since it's real hardware:

- `MIDI_INPUT_PORT` — the USB controller's input port name, matched by substring (e.g.
  `MIDI_INPUT_PORT=X-Session` matches any port name containing that text).
  - macOS default: `USB X-Session Anschluss 1`.
  - Windows: the X-Session Pro is class-compliant — no driver install needed. It enumerates as
    `USB X-Session` (confirmed on Windows 11 24H2+/build 26200).
  - In general, if a name doesn't match, enumerate available ports by running the binary once
    with no env var set — it prints all available ports it couldn't match.

# Usage

macOS:
```
$ MIDI_INPUT_PORT="USB X-Session Anschluss 1" OSC_HOST=127.0.0.1 OSC_PORT=7000 ./target/release/midi-auto-crossfader
```

Windows (verified working):
```
> $env:MIDI_INPUT_PORT="USB X-Session"; $env:OSC_HOST="127.0.0.1"; $env:OSC_PORT="7000"; .\target\release\midi-auto-crossfader.exe
```

Note: the `Crossfader A`/`Crossfader B` Windows MIDI Services loopback pair from earlier setup is
no longer needed now that output is OSC — it was only required for MIDI output. The USB
controller input side needs no virtual port at all, just its own class-compliant driver.

- Default Duration: `10 seconds`
- Fade to Left: `Ctrl` + `<`, or the USB controller's `⏴` button (lower left)
- Fade to Right: `Ctrl` + `Y`, or the USB controller's `⏵` button (lower left)
- Decrease Duration: `Ctrl` + `A`
- Increase Duration: `Ctrl` + `S`
- Quit: `Esc`
- Moving the USB controller's physical crossfader manually aborts any running auto-crossfade
  and adopts its current position as the new starting value.

Keyboard shortcuts are global (work while any app is focused) — on macOS this requires granting
Accessibility permissions to the terminal/binary; on Windows it may require running as the same
user session as the target app (no elevation needed for normal use).

Arrow keys are deliberately not used for shortcuts: `rdev`'s global hook only *observes*
keystrokes, it doesn't consume them, so the terminal window still receives them too - and many
terminals bind `Ctrl`+arrows to scrolling the buffer, which hides the app's pinned status lines.
`Ctrl`+`<`/`Y`/`A`/`S` are matched on the physical key position (not the character produced),
since holding `Ctrl` makes the OS report a control character instead of the plain letter.
