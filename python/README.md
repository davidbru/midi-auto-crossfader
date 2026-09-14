# Install
```
$ pip install mido python-rtmidi
$ pip install pynput
```


# Usage
```
$ python crossfade.py
```
- Default Duration
  - `10 Seconds`
- Fade to Left
  - MacBook `"CTRL" + "<"`
  - USB Midicontroller `⏴` in the lower left
- Fade to Right
  - MacBook `"CTRL" + "y"`
  - USB Midicontroller `⏵` in the lower left
- Decrease Duration
  - MacBook `"CTRL" + "Arrow Left"`
- Increase Duration
  - MacBook `"CTRL" + "Arrow Right"`

Note: this version is macOS-only as written — it hardcodes the `IAC-Treiber Bus 1` virtual
MIDI bus (macOS's built-in IAC Driver) and the `USB X-Session Anschluss 1` port name that
macOS assigns to the controller. See [`../rust/`](../rust/) for the cross-platform rewrite.
