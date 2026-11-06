use keysteel_daemon::lights::LightEngine;
use keysteel_hid::HidrawLights;
use std::sync::{Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

/// Breathing changes slowly, so fifteen steps a second look smooth.
const STEP_INTERVAL: Duration = Duration::from_millis(66);

pub struct LightInner {
    pub engine: LightEngine<HidrawLights>,
    /// When the scheme or its settings last changed; the engine's clock.
    pub started: Instant,
}

/// The lighting counterpart of `Shared`: state shared by the lighting loop,
/// the D-Bus service and the hotplug watcher.
pub struct LightShared {
    inner: Mutex<LightInner>,
    wake: Condvar,
}

impl LightShared {
    pub fn new(engine: LightEngine<HidrawLights>) -> LightShared {
        LightShared { inner: Mutex::new(LightInner { engine, started: Instant::now() }), wake: Condvar::new() }
    }

    pub fn lock(&self) -> MutexGuard<'_, LightInner> {
        self.inner.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Tells the lighting loop that the scheme or the keyboard changed.
    pub fn wake(&self) {
        self.wake.notify_all();
    }

    /// Steps an animated scheme. Static ones block here until woken, so an
    /// idle daemon uses no timer.
    pub fn run(&self) -> ! {
        let mut guard = self.lock();
        loop {
            if guard.engine.needs_ticks() {
                let elapsed = guard.started.elapsed();
                guard.engine.tick(elapsed);
                guard = self
                    .wake
                    .wait_timeout(guard, STEP_INTERVAL)
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .0;
            } else {
                guard = self.wake.wait(guard).unwrap_or_else(|poisoned| poisoned.into_inner());
            }
        }
    }
}
