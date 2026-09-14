# midi-auto-crossfader

Listens for button presses on an M-Audio X-Session Pro USB MIDI controller (or a global
keyboard shortcut) and automatically ramps a MIDI Control Change value from one extreme to the
other over a configurable duration — used to crossfade between two video/visual sources driven
by MIDI CC.

Two implementations live here:

- [`rust/`](rust/) — the current, cross-platform (macOS + Windows) version. Start here.
- [`python/`](python/) — the original macOS-only implementation, kept for reference.
