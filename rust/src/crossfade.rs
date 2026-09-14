use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::config::{DEFAULT_DURATION_INDEX, DURATIONS};
use crate::osc::OscOutput;
use crate::ui::Ui;

const TICK: Duration = Duration::from_millis(20); // 50Hz - smooth for visuals, light on the network
// The terminal UI redraws far less often than OSC sends: hammering the console at 50Hz from
// this thread while another thread (e.g. a keyboard shortcut) also writes to it causes visible
// lag, and a moving text marker doesn't need anywhere near 50Hz to look smooth anyway.
const UI_UPDATE_EVERY_N_TICKS: u32 = 4; // ~12.5Hz

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    Left,
    Right,
}

impl Direction {
    fn arrow(self) -> char {
        match self {
            Direction::Left => '<',
            Direction::Right => '>',
        }
    }
}

pub struct CrossfadeState {
    running: AtomicBool,
    interrupt: AtomicBool,
    direction: Mutex<Option<Direction>>,
    value: Mutex<f32>, // current crossfade value, 0.0-1.0 (0.5 = middle)
    duration_index: AtomicUsize,
    thread: Mutex<Option<JoinHandle<()>>>,
    osc: OscOutput,
    ui: Ui,
}

impl CrossfadeState {
    pub fn new(osc: OscOutput, ui: Ui) -> Self {
        Self {
            running: AtomicBool::new(false),
            interrupt: AtomicBool::new(false),
            direction: Mutex::new(None),
            value: Mutex::new(0.5),
            duration_index: AtomicUsize::new(DEFAULT_DURATION_INDEX),
            thread: Mutex::new(None),
            osc,
            ui,
        }
    }

    /// Logs a message above the pinned status lines - use this instead of println!
    /// anywhere that has access to the state, so output never corrupts the status display.
    pub fn log(&self, msg: impl AsRef<str>) {
        self.ui.log(msg);
    }

    pub fn finish_ui(&self) {
        self.ui.finish();
    }

    pub fn set_value(&self, value: f32) {
        let value = value.clamp(0.0, 1.0);
        *self.value.lock().unwrap() = value;
        self.ui.set_position(value, None);
    }

    pub fn adjust_duration(&self, increase: bool) {
        let mut idx = self.duration_index.load(Ordering::SeqCst);
        if increase && idx < DURATIONS.len() - 1 {
            idx += 1;
        } else if !increase && idx > 0 {
            idx -= 1;
        }
        self.duration_index.store(idx, Ordering::SeqCst);
        self.ui.set_duration(DURATIONS[idx]);
    }

    /// Starts crossfading in `direction`. If already running in the opposite
    /// direction, interrupts and waits for that thread before switching.
    pub fn start(self: &Arc<Self>, direction: Direction) {
        let mut dir_guard = self.direction.lock().unwrap();
        let running = self.running.load(Ordering::SeqCst);

        if running && *dir_guard == Some(direction) {
            self.log(format!("Already crossfading {direction:?}, skipping redundant start."));
            return;
        }

        if running {
            self.log(format!("Interrupting crossfade to switch direction to {direction:?}"));
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
        let mut tick_count: u32 = 0;
        // Scheduled against an absolute clock, not a relative sleep(TICK) after each
        // iteration: a relative sleep means any slow iteration (e.g. a terminal redraw
        // contending with a keypress) permanently pushes back every later tick too, which
        // compounds into multi-second drift over a long-running fade. Targeting fixed
        // points in time instead lets a late tick just get a shorter sleep next time,
        // self-correcting instead of accumulating delay.
        let mut next_tick = Instant::now();

        loop {
            if self.interrupt.load(Ordering::SeqCst) {
                let current = *self.value.lock().unwrap();
                self.ui.set_position(current, None);
                self.log("Crossfade interrupted!");
                return;
            }

            // Re-read the duration every tick (rather than once at thread start) so that
            // changing it mid-fade takes effect immediately: the remaining distance is
            // covered at the new duration's rate, instead of finishing at the old rate.
            let total_duration = DURATIONS[self.duration_index.load(Ordering::SeqCst)] as f32;
            let step = TICK.as_secs_f32() / total_duration;

            let direction = *self.direction.lock().unwrap();
            let mut value = self.value.lock().unwrap();

            let next = match direction {
                Some(Direction::Left) if *value > 0.0 => (*value - step).max(0.0),
                Some(Direction::Right) if *value < 1.0 => (*value + step).min(1.0),
                _ => {
                    self.running.store(false, Ordering::SeqCst);
                    self.ui.set_position(*value, None);
                    return;
                }
            };

            *value = next;
            drop(value);
            self.osc.send(next);

            if tick_count % UI_UPDATE_EVERY_N_TICKS == 0 {
                self.ui.set_position(next, Some(direction.unwrap().arrow()));
            }
            tick_count = tick_count.wrapping_add(1);

            next_tick += TICK;
            let now = Instant::now();
            if next_tick > now {
                thread::sleep(next_tick - now);
            } else {
                // Running behind schedule - catch back up to "now" instead of trying to
                // burn through a backlog of already-missed ticks one by one.
                next_tick = now;
            }
        }
    }
}
