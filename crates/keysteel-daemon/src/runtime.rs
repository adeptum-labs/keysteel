use keysteel_daemon::engine::Engine;
use keysteel_hid::HidrawDevice;
use std::sync::{Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

const FRAME_INTERVAL: Duration = Duration::from_millis(33);

pub struct Inner {
    pub engine: Engine<HidrawDevice>,
    /// When the current scene started; the engine's clock.
    pub started: Instant,
}

/// State shared by the render loop, the D-Bus service and the hotplug watcher.
pub struct Shared {
    inner: Mutex<Inner>,
    wake: Condvar,
}

impl Shared {
    pub fn new(engine: Engine<HidrawDevice>) -> Shared {
        Shared {
            inner: Mutex::new(Inner { engine, started: Instant::now() }),
            wake: Condvar::new(),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Tells the render loop that scene or device changed.
    pub fn wake(&self) {
        self.wake.notify_all();
    }

    /// Draws frames while an animated scene is shown. When there is nothing
    /// to animate it blocks until woken, so an idle daemon uses no timer.
    pub fn render_loop(&self) -> ! {
        let mut guard = self.lock();
        loop {
            if guard.engine.needs_ticks() {
                let elapsed = guard.started.elapsed();
                guard.engine.tick(elapsed);
                guard = self
                    .wake
                    .wait_timeout(guard, FRAME_INTERVAL)
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .0;
            } else {
                guard = self.wake.wait(guard).unwrap_or_else(|poisoned| poisoned.into_inner());
            }
        }
    }
}
