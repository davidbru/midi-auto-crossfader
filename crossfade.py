from pynput import keyboard
import mido
import time

# Specify the Control Change number and values for the crossfader
MIDI_CC_NUMBER = 60   # CC number for Composition Crossfader Phase
MIDI_CC_VALUE  = 0    # Value (0-127) where 64 is the middle position (0 in range -1 to 1)
MIDI_CHANNEL   = 1    # MIDI channel (1-based)

CTRL_PRESSED = False



def on_press(KEY):
    global CTRL_PRESSED

    try:
        if KEY == keyboard.Key.ctrl_l or KEY == keyboard.Key.ctrl_r:
            CTRL_PRESSED = True

        if CTRL_PRESSED:
            if KEY.char == '<':
                print("Ctrl + < detected!")
                with mido.open_output('IAC-Treiber Bus 1') as OUTPUT_PORT:
                    send_midi_cc(0, OUTPUT_PORT)

            if KEY.char == 'z':
                print("Ctrl + Y detected!")
                with mido.open_output('IAC-Treiber Bus 1') as OUTPUT_PORT:
                    send_midi_cc(127, OUTPUT_PORT)

    except AttributeError:
        pass  # Handle special keys like arrows



def on_release(KEY):
    global CTRL_PRESSED

    # If Ctrl is released
    if KEY == keyboard.Key.ctrl_l or KEY == keyboard.Key.ctrl_r:
        CTRL_PRESSED = False

    # Stop listener if Esc is pressed
    if KEY == keyboard.Key.esc:
        return False



def send_midi_cc(VALUE, OUTPUT_PORT):
    msg_cc = mido.Message('control_change', control=MIDI_CC_NUMBER, value=VALUE, channel=MIDI_CHANNEL-1)
    OUTPUT_PORT.send(msg_cc)
    print(f'Sent Control Change: {msg_cc}')




with keyboard.Listener(on_press=on_press, on_release=on_release) as listener:
    listener.join()