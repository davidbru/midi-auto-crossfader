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

    while crossfade_running:
        if crossfade_interrupt:
            # Exit if interrupted
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

        time.sleep(0.05)  # Delay for smooth crossfading

# Start crossfade in a separate thread, with interruption logic
def start_crossfade(direction):
    global crossfade_running, crossfade_direction, crossfade_thread, crossfade_interrupt

    if crossfade_running and crossfade_direction != direction:
        # If already running, interrupt and switch direction
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

# Function to handle key press events
def on_press(key):
    global ctrl_pressed

    try:
        if key == keyboard.Key.ctrl_l or key == keyboard.Key.ctrl_r:
            ctrl_pressed = True

        if ctrl_pressed:
            if key.char == '<':
                print("Ctrl + < detected: Crossfade down")
                start_crossfade('down')

            if key.char == 'z':
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