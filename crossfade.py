from pynput import keyboard
import mido
import time

# Specify the Control Change number and values for the crossfader
MIDI_CC_NUMBER         = 60   # CC number for Composition Crossfader Phase
MIDI_CURRENT_VALUE     = 64   # Value (0-127) where 64 is the middle position (0 in range -1 to 1)
MIDI_CHANNEL           = 1    # MIDI channel (1-based)

CTRL_PRESSED           = False
CROSSFADE_RUNNING_DOWN = False
CROSSFADE_COUNT_DIRECTION = False


# Open the MIDI output port once on startup
OUTPUT_PORT = mido.open_output('IAC-Treiber Bus 1')

def on_press(KEY):
    global CTRL_PRESSED

    try:
        if KEY == keyboard.Key.ctrl_l or KEY == keyboard.Key.ctrl_r:
            CTRL_PRESSED = True

        if CTRL_PRESSED:
            if KEY.char == '<':
                print("Ctrl + < detected!")
                decrease_value(True)

            if KEY.char == 'z':
                print("Ctrl + Y detected!")
                increase_value(True)

    except AttributeError:
        pass  # Handle special keys like arrows



def on_release(KEY):
    global CTRL_PRESSED

    # If Ctrl is released
    if KEY == keyboard.Key.ctrl_l or KEY == keyboard.Key.ctrl_r:
        CTRL_PRESSED = False

    # Stop listener if Esc is pressed
    if KEY == keyboard.Key.esc:
        OUTPUT_PORT.close()
        return False



def send_midi_cc(VALUE, CONTEXT):
    print(f'      send_midi_cc({VALUE}, {CONTEXT})')
    msg_cc = mido.Message('control_change', control=MIDI_CC_NUMBER, value=VALUE, channel=MIDI_CHANNEL-1)
    OUTPUT_PORT.send(msg_cc)
    print(f'      Sent Control Change: {msg_cc}, {CONTEXT}')



def decrease_value(IS_INITIAL_CALL = False):
    global CROSSFADE_COUNT_DIRECTION, MIDI_CURRENT_VALUE
    print(f'  decrease_value({IS_INITIAL_CALL}): {CROSSFADE_COUNT_DIRECTION}, {MIDI_CURRENT_VALUE}')

    if IS_INITIAL_CALL == True:
        CROSSFADE_COUNT_DIRECTION = 'down'

    if CROSSFADE_COUNT_DIRECTION == 'down':
        print(f'    decrease_value - before send_midi_cc({MIDI_CURRENT_VALUE}, down)')
        send_midi_cc(MIDI_CURRENT_VALUE, 'down')
        time.sleep(0.1)

        if MIDI_CURRENT_VALUE > 0:
            MIDI_CURRENT_VALUE = MIDI_CURRENT_VALUE - 1
            decrease_value()

        elif MIDI_CURRENT_VALUE == 0:
            CROSSFADE_COUNT_DIRECTION = False
            print("Decreasing completed, MIDI value is now 0.")
            return



def increase_value(IS_INITIAL_CALL = False):
    global CROSSFADE_COUNT_DIRECTION, MIDI_CURRENT_VALUE
    print(f'  increase_value({IS_INITIAL_CALL}): {CROSSFADE_COUNT_DIRECTION}, {MIDI_CURRENT_VALUE}')

    if IS_INITIAL_CALL == True:
        CROSSFADE_COUNT_DIRECTION = 'up'

    if CROSSFADE_COUNT_DIRECTION == 'up':
        print(f'    increase_value - before send_midi_cc({MIDI_CURRENT_VALUE}, up)')
        send_midi_cc(MIDI_CURRENT_VALUE, 'up')
        time.sleep(0.1)

        if MIDI_CURRENT_VALUE < 126:
            MIDI_CURRENT_VALUE = MIDI_CURRENT_VALUE + 1
            increase_value()

        elif MIDI_CURRENT_VALUE == 126:
            CROSSFADE_COUNT_DIRECTION = False
            print("Increasing completed, MIDI value is now 126.")
            return


with keyboard.Listener(on_press=on_press, on_release=on_release) as listener:
    listener.join()


OUTPUT_PORT.close()