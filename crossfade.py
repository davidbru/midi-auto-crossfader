from pynput import keyboard
import mido
import threading
import time

# MIDI CC details
MIDI_CC_NUMBER = 60   # CC number for Composition Crossfader Phase
MIDI_CHANNEL = 1    # MIDI channel (1-based)

midi_current_value = 64   # Current value (0-127), where 64 is the middle
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

# Function to send MIDI Control Change message
def send_midi_cc(value, direction):
    print(f'Sending MIDI CC {value}, direction: {direction}')
    msg_cc = mido.Message('control_change', control=MIDI_CC_NUMBER, value=value, channel=MIDI_CHANNEL-1)
    OUTPUT_PORT.send(msg_cc)
    print(f'Sent Control Change: {msg_cc}')

# Function to handle crossfade logic in a separate thread
def crossfade_loop():
    global midi_current_value, crossfade_running, crossfade_direction, crossfade_interrupt

    total_duration = DURATIONS[duration_index]  # Get the total duration
    delay = total_duration / 127  # Calculate the delay between each step

    while crossfade_running:
        if crossfade_interrupt:
            print("Crossfade interrupted!")
            return

        if crossfade_direction == 'down' and midi_current_value > 0:
            midi_current_value -= 1
            send_midi_cc(midi_current_value, 'down')

        elif crossfade_direction == 'up' and midi_current_value < 127:
            midi_current_value += 1
            send_midi_cc(midi_current_value, 'up')

        else:
            crossfade_running = False
            return

        time.sleep(delay)  # Dynamic delay based on duration

# Start crossfade in a separate thread, with interruption logic
def start_crossfade(direction):
    global crossfade_running, crossfade_direction, crossfade_thread, crossfade_interrupt

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
                print("Ctrl + Arrow Right detected: Increase duration")
                adjust_duration(increase=True)

            elif key == keyboard.Key.left:
                print("Ctrl + Arrow Left detected: Decrease duration")
                adjust_duration(increase=False)

            elif key.char == '<':
                print("Ctrl + < detected: Crossfade down")
                start_crossfade('down')

            elif key.char == 'z':
                print("Ctrl + Z detected: Crossfade up")
                start_crossfade('up')

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
        return False

# Start the keyboard listener
with keyboard.Listener(on_press=on_press, on_release=on_release) as listener:
    listener.join()

OUTPUT_PORT.close()
