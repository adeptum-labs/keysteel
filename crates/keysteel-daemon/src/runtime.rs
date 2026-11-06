// SPDX-License-Identifier: GPL-3.0-or-later
//
// Copyright © 2026 Adam Waldenberg and Adeptum AB, Org.nr 559494-1824
//
// This file is part of Keysteel.
//
// Keysteel is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option)
// any later version.
//
// Keysteel is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
// FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for
// more details.
//
// You should have received a copy of the GNU General Public License along
// with Keysteel. If not, see <https://www.gnu.org/licenses/>.

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
