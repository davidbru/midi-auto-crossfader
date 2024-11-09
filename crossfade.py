from pynput import keyboard
import mido
import threading
import time

# MIDI CC details
MIDI_CC_NUMBER = 60                 # CC number for Composition Crossfader Phase
MIDI_CHANNEL = 1                    # MIDI channel (1-based)
MIDICONTROLLER_LEFT_CC_NUMBER = 87  # CC number for the "Fade to Left"-button on USB X-Session Anschluss 1
MIDICONTROLLER_RIGHT_CC_NUMBER = 15 # CC number for the "Fade to Right"-button on USB X-Session Anschluss 1

# DEFAULT VALUES
midi_current_value = 64 # Current value (0-127), where 64 is the middle
ctrl_pressed = False
crossfade_thread = None
crossfade_running = False
crossfade_direction = None
crossfade_interrupt = False

# Durations for total crossfade time (from 0 to 127)
DURATIONS = [1, 2, 10, 30, 60, 300, 600]  # In seconds: 1sec, 2sec, 10sec, 30sec, 1min, 5min, 10min
duration_index = 2  # Start with the 10sec duration by default

# Open MIDI output once
OUTPUT_PORT = mido.open_output('IAC-Treiber Bus 1')
USB_XSESSION_PORT = None

try:
    USB_XSESSION_PORT = mido.open_input('USB X-Session Anschluss 1')
except IOError:
    print("'USB X-Session Anschluss 1' not available")

# Function to send MIDI Control Change message
def send_midi_cc(value, direction):
    print(f'Sending MIDI CC {value}, direction: {direction}')
    msg_cc = mido.Message('control_change', control=MIDI_CC_NUMBER, value=value, channel=MIDI_CHANNEL-1)
    OUTPUT_PORT.send(msg_cc)
    # print(f'Sent Control Change: {msg_cc}')

# Function to handle crossfade logic in a separate thread
def crossfade_loop():
    global midi_current_value, crossfade_running, crossfade_direction, crossfade_interrupt

    total_duration = DURATIONS[duration_index]  # Get the total duration
    delay = total_duration / 127  # Calculate the delay between each step

    while crossfade_running:
        if crossfade_interrupt:
            print("Crossfade interrupted!")
            return

        if crossfade_direction == 'left' and midi_current_value > 0:
            midi_current_value -= 1
            send_midi_cc(midi_current_value, 'left')

        elif crossfade_direction == 'right' and midi_current_value < 127:
            midi_current_value += 1
            send_midi_cc(midi_current_value, 'right')

        else:
            crossfade_running = False
            return

        time.sleep(delay)  # Dynamic delay based on duration

# Start crossfade in a separate thread, with interruption logic
def start_crossfade(direction):
    global crossfade_running, crossfade_direction, crossfade_thread, crossfade_interrupt

    # If already running in the same direction, don't start another thread
    if crossfade_running and crossfade_direction == direction:
        print(f"Already crossfading {direction}, skipping redundant start.")
        return

    # If running in the opposite direction, interrupt and wait for the previous thread to stop
    if crossfade_running and crossfade_direction != direction:
        print(f"Interrupting crossfade to switch direction to {direction}")
        crossfade_interrupt = True
        crossfade_thread.join()  # Wait for the previous thread to stop

    # Start new crossfade
    crossfade_direction = direction
    crossfade_running = True
    crossfade_interrupt = False
    crossfade_thread = threading.Thread(target=crossfade_loop)
    crossfade_thread.start()

# Stop the crossfade thread
def stop_crossfade():
    global crossfade_running, crossfade_interrupt
    crossfade_interrupt = True
    crossfade_running = False

# Adjust duration
def adjust_duration(increase=True):
    global duration_index

    if increase and duration_index < len(DURATIONS) - 1:
        duration_index += 1
    elif not increase and duration_index > 0:
        duration_index -= 1

    print(f"Duration set to {DURATIONS[duration_index]} seconds")

# Function to handle key press events
def on_press(key):
    global ctrl_pressed

    try:
        if key == keyboard.Key.ctrl_l or key == keyboard.Key.ctrl_r:
            ctrl_pressed = True

        if ctrl_pressed:
            if key == keyboard.Key.right:
                print("[Keyboard] Ctrl + Arrow Right pressed: Increase duration")
                adjust_duration(increase=True)

            elif key == keyboard.Key.left:
                print("[Keyboard] Ctrl + Arrow Left pressed: Decrease duration")
                adjust_duration(increase=False)

            elif key.char == '<':
                print("[Keyboard] Ctrl + < pressed: Crossfade left")
                start_crossfade('left')

            elif key.char == 'z':
                print("[Keyboard] Ctrl + Y pressed: Crossfade right")
                start_crossfade('right')

    except AttributeError:
        pass  # Special keys like arrows

# Function to handle key release events
def on_release(key):
    global ctrl_pressed

    if key == keyboard.Key.ctrl_l or key == keyboard.Key.ctrl_r:
        ctrl_pressed = False

    if key == keyboard.Key.esc:
        stop_crossfade()
        OUTPUT_PORT.close()
        if USB_XSESSION_PORT:
            USB_XSESSION_PORT.close()
        return False

# Function to listen for MIDI messages on USB X-Session
def midi_listener():
    if USB_XSESSION_PORT:
        for msg in USB_XSESSION_PORT:
            if msg.type == 'control_change' and msg.control == MIDICONTROLLER_LEFT_CC_NUMBER:
                print("[USB Controller] ⏴ pressed: Crossfade left")
                start_crossfade('left')
            if msg.type == 'control_change' and msg.control == MIDICONTROLLER_RIGHT_CC_NUMBER:
                print("[USB Controller] ⏵ pressed: Crossfade right")
                start_crossfade('right')

# Start the MIDI listener thread
if USB_XSESSION_PORT:
    print("USB_XSESSION_PORT available")
    threading.Thread(target=midi_listener, daemon=True).start()
else:
    print("USB_XSESSION_PORT NOT available")

# Start the keyboard listener
with keyboard.Listener(on_press=on_press, on_release=on_release) as listener:
    listener.join()

OUTPUT_PORT.close()
if USB_XSESSION_PORT:
    USB_XSESSION_PORT.close()
