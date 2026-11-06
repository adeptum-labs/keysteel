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

use crate::light_runtime::LightShared;
use crate::runtime::Shared;
use crate::scroll::Roller;
use keysteel_core::lighting::{all_light_schemes, LightParams, Rgb};
use keysteel_core::scene::all_scenes;
use keysteel_daemon::state;
use keysteel_hid::{HidrawDevice, HidrawLights};
use std::sync::Arc;
use std::time::Instant;
use zbus::{fdo, interface};

pub const BUS_NAME: &str = "org.keysteel.Daemon";
pub const OBJECT_PATH: &str = "/org/keysteel/Daemon";

pub struct Service {
    pub shared: Arc<Shared>,
    pub lights: Arc<LightShared>,
    pub roller: Arc<Roller>,
}

#[interface(name = "org.keysteel.Daemon")]
impl Service {
    /// All scenes as (id, display name).
    fn list_scenes(&self) -> Vec<(String, String)> {
        all_scenes()
            .iter()
            .map(|scene| (scene.id().to_string(), scene.name().to_string()))
            .collect()
    }

    fn set_scene(&self, id: &str) -> fdo::Result<()> {
        {
            let mut inner = self.shared.lock();
            // A keyboard dropped after a failed write is picked up again here,
            // so changing the setting is also the way to get it back.
            if inner.engine.device().is_none() {
                if let Ok(device) = HidrawDevice::open() {
                    inner.engine.set_device(Some(device));
                }
            }
            inner
                .engine
                .set_scene(id)
                .map_err(|e| fdo::Error::InvalidArgs(e.to_string()))?;
            inner.started = Instant::now();
            // Saved while the lock is held, so the file always matches the
            // scene that was applied last.
            if let Some(path) = state::default_path() {
                if let Err(e) = state::save_to(&path, id) {
                    eprintln!("cannot save scene: {e}");
                }
            }
        }
        self.shared.wake();
        Ok(())
    }

    fn get_scene(&self) -> String {
        self.shared.lock().engine.current().to_string()
    }

    /// All lighting schemes as (id, name, takes a colour, uses the speed,
    /// uses the intensity).
    fn list_light_schemes(&self) -> Vec<(String, String, bool, bool, bool)> {
        all_light_schemes()
            .iter()
            .map(|s| (s.id().to_string(), s.name().to_string(), s.takes_colour(), s.uses_speed(), s.uses_intensity()))
            .collect()
    }

    /// The scheme id, intensity (0 to 100), red, green, blue and speed (1 to 10).
    fn get_lighting(&self) -> (String, u8, u8, u8, u8, u8) {
        let inner = self.lights.lock();
        let p = inner.engine.params();
        (inner.engine.scheme_id().to_string(), p.intensity, p.colour.r, p.colour.g, p.colour.b, p.speed)
    }

    fn set_lighting(&self, scheme: &str, intensity: u8, red: u8, green: u8, blue: u8, speed: u8) -> fdo::Result<()> {
        let params = LightParams { intensity, colour: Rgb::new(red, green, blue), speed };
        {
            let mut inner = self.lights.lock();
            if inner.engine.lights().is_none() {
                if let Ok(handle) = HidrawLights::open() {
                    inner.engine.set_device(Some(handle));
                }
            }
            inner
                .engine
                .set(scheme, params)
                .map_err(|e| fdo::Error::InvalidArgs(e.to_string()))?;
            inner.started = Instant::now();
            // Saved under the lock so the file matches what was applied last.
            if let Some(path) = state::lighting_path() {
                if let Err(e) = state::save_lighting(&path, scheme, &inner.engine.params()) {
                    eprintln!("cannot save lighting: {e}");
                }
            }
        }
        self.lights.wake();
        Ok(())
    }

    /// Whether the roller scrolls the window instead of changing the volume.
    fn get_roller_scroll(&self) -> bool {
        self.roller.get()
    }

    fn set_roller_scroll(&self, on: bool) {
        self.roller.set(on);
        if let Some(path) = state::roller_path() {
            if let Err(e) = state::save_roller(&path, on) {
                eprintln!("cannot save roller setting: {e}");
            }
        }
    }

    /// Wheel notches for each click of the roller, from 1 to 10.
    fn get_roller_speed(&self) -> u32 {
        self.roller.speed()
    }

    fn set_roller_speed(&self, speed: u32) {
        self.roller.set_speed(speed);
        if let Some(path) = state::speed_path() {
            if let Err(e) = state::save_speed(&path, self.roller.speed()) {
                eprintln!("cannot save roller speed: {e}");
            }
        }
    }
}
