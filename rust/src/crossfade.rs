use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use midir::MidiOutputConnection;

use crate::config::{DEFAULT_DURATION_INDEX, DURATIONS, MIDI_CC_NUMBER, MIDI_CHANNEL};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    Left,
    Right,
}

pub struct CrossfadeState {
    running: AtomicBool,
    interrupt: AtomicBool,
    direction: Mutex<Option<Direction>>,
    value: AtomicU8, // current crossfade value, 0-127 (64 = middle)
    duration_index: AtomicUsize,
    thread: Mutex<Option<JoinHandle<()>>>,
    output: Mutex<MidiOutputConnection>,
}

impl CrossfadeState {
    pub fn new(output: MidiOutputConnection) -> Self {
        Self {
            running: AtomicBool::new(false),
            interrupt: AtomicBool::new(false),
            direction: Mutex::new(None),
            value: AtomicU8::new(64),
            duration_index: AtomicUsize::new(DEFAULT_DURATION_INDEX),
            thread: Mutex::new(None),
            output: Mutex::new(output),
        }
    }

    pub fn set_value(&self, value: u8) {
        self.value.store(value, Ordering::SeqCst);
    }

    pub fn adjust_duration(&self, increase: bool) {
        let mut idx = self.duration_index.load(Ordering::SeqCst);
        if increase && idx < DURATIONS.len() - 1 {
            idx += 1;
        } else if !increase && idx > 0 {
            idx -= 1;
        }
        self.duration_index.store(idx, Ordering::SeqCst);
        println!("Duration set to {} seconds", DURATIONS[idx]);
    }

    /// Starts crossfading in `direction`. If already running in the opposite
    /// direction, interrupts and waits for that thread before switching.
    pub fn start(self: &Arc<Self>, direction: Direction) {
        let mut dir_guard = self.direction.lock().unwrap();
        let running = self.running.load(Ordering::SeqCst);

        if running && *dir_guard == Some(direction) {
            println!("Already crossfading {direction:?}, skipping redundant start.");
            return;
        }

        if running {
            println!("Interrupting crossfade to switch direction to {direction:?}");
            self.interrupt.store(true, Ordering::SeqCst);
            drop(dir_guard); // don't hold the lock while joining the other thread
            if let Some(handle) = self.thread.lock().unwrap().take() {
                let _ = handle.join();
            }
            dir_guard = self.direction.lock().unwrap();
        }

        *dir_guard = Some(direction);
        drop(dir_guard);
        self.interrupt.store(false, Ordering::SeqCst);
        self.running.store(true, Ordering::SeqCst);

        let state = Arc::clone(self);
        let handle = thread::spawn(move || state.run_loop());
        *self.thread.lock().unwrap() = Some(handle);
    }

    pub fn stop(&self) {
        self.interrupt.store(true, Ordering::SeqCst);
        self.running.store(false, Ordering::SeqCst);
    }

    fn run_loop(&self) {
        let total_duration = DURATIONS[self.duration_index.load(Ordering::SeqCst)];
        let delay = Duration::from_secs_f64(total_duration as f64 / 127.0);

        loop {
            if self.interrupt.load(Ordering::SeqCst) {
                println!("Crossfade interrupted!");
                return;
            }

            let direction = *self.direction.lock().unwrap();
            let current = self.value.load(Ordering::SeqCst);

            let next = match direction {
                Some(Direction::Left) if current > 0 => current - 1,
                Some(Direction::Right) if current < 127 => current + 1,
                _ => {
                    self.running.store(false, Ordering::SeqCst);
                    return;
                }
            };

            self.value.store(next, Ordering::SeqCst);
            self.send(next, direction.unwrap());

            thread::sleep(delay);
        }
    }

    fn send(&self, value: u8, direction: Direction) {
        let status = 0xB0 | (MIDI_CHANNEL - 1);
        let result = self
            .output
            .lock()
            .unwrap()
            .send(&[status, MIDI_CC_NUMBER, value]);
        match result {
            Ok(()) => println!("Sending MIDI CC {value}, direction: {direction:?}"),
            Err(e) => eprintln!("[MIDI Output] failed to send CC: {e}"),
        }
    }
}
