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

use keysteel_core::lighting::{light_scheme_by_id, LightParams, LightScheme, Lighting};
use keysteel_hid::KeyLights;
use std::time::Duration;

#[derive(Debug, PartialEq, Eq)]
pub struct UnknownLightScheme(pub String);

impl std::fmt::Display for UnknownLightScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unknown lighting scheme: {}", self.0)
    }
}

impl std::error::Error for UnknownLightScheme {}

/// Keeps the chosen lighting scheme and its settings, and shows them on the
/// keyboard if one is attached.
///
/// `elapsed` is measured from the moment the scheme or its settings last
/// changed; the caller owns the clock.
pub struct LightEngine<L: KeyLights> {
    lights: Option<L>,
    scheme: Box<dyn LightScheme>,
    params: LightParams,
    last_elapsed: Duration,
    /// What the keyboard shows now, so an unchanged result is not sent again.
    shown: Option<Lighting>,
}

impl<L: KeyLights> LightEngine<L> {
    /// Starts on `scheme_id` (or the keyboard's own lighting when unknown).
    pub fn new(lights: Option<L>, scheme_id: &str, params: LightParams) -> LightEngine<L> {
        let scheme = light_scheme_by_id(scheme_id)
            .or_else(|| light_scheme_by_id("default"))
            .expect("the default scheme always exists");
        let mut engine = LightEngine {
            lights,
            scheme,
            params: params.clamped(),
            last_elapsed: Duration::ZERO,
            shown: None,
        };
        engine.present();
        engine
    }

    pub fn scheme_id(&self) -> &str {
        self.scheme.id()
    }

    pub fn params(&self) -> LightParams {
        self.params
    }

    pub fn lights(&self) -> Option<&L> {
        self.lights.as_ref()
    }

    /// True when the scheme changes over time and there is a keyboard to light.
    pub fn needs_ticks(&self) -> bool {
        self.scheme.animated() && self.lights.is_some()
    }

    /// Switches scheme and settings and shows the result. An unknown scheme
    /// changes nothing.
    pub fn set(&mut self, scheme_id: &str, params: LightParams) -> Result<(), UnknownLightScheme> {
        let scheme = light_scheme_by_id(scheme_id).ok_or_else(|| UnknownLightScheme(scheme_id.to_string()))?;
        self.scheme = scheme;
        self.params = params.clamped();
        self.last_elapsed = Duration::ZERO;
        self.present();
        Ok(())
    }

    /// Shows the next step of an animated scheme.
    pub fn tick(&mut self, elapsed: Duration) {
        self.last_elapsed = elapsed;
        if self.scheme.animated() {
            self.present();
        }
    }

    /// Attaches or detaches the keyboard and shows the current lighting on it.
    pub fn set_device(&mut self, lights: Option<L>) {
        self.lights = lights;
        self.shown = None;
        self.present();
    }

    fn present(&mut self) {
        let Some(lights) = self.lights.as_mut() else {
            return;
        };
        let wanted = self.scheme.render(self.last_elapsed, &self.params);
        if self.shown == Some(wanted) {
            return;
        }
        let result = match wanted {
            Lighting::All(colour) => lights.set_all(colour),
            Lighting::HandBack => lights.clear_keys(),
        };
        match result {
            Ok(()) => self.shown = Some(wanted),
            Err(e) => {
                eprintln!("lighting write failed, dropping the keyboard: {e}");
                self.lights = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keysteel_core::lighting::Rgb;
    use keysteel_hid::{DeviceError, KeyColour, MockLights};

    struct FailingLights;

    impl KeyLights for FailingLights {
        fn set_keys(&mut self, _keys: &[KeyColour]) -> Result<(), DeviceError> {
            Err(DeviceError::Hid("unplugged".into()))
        }

        fn clear_keys(&mut self) -> Result<(), DeviceError> {
            Err(DeviceError::Hid("unplugged".into()))
        }
    }

    fn engine(scheme: &str) -> LightEngine<MockLights> {
        LightEngine::new(Some(MockLights::default()), scheme, LightParams::default())
    }

    fn last_colour(engine: &LightEngine<MockLights>) -> Rgb {
        engine.lights().unwrap().sets.last().unwrap()[0].colour
    }

    #[test]
    fn starting_on_default_hands_the_lighting_back() {
        let engine = engine("default");
        let lights = engine.lights().unwrap();
        assert_eq!(lights.clears, 1);
        assert!(lights.sets.is_empty());
    }

    #[test]
    fn unknown_scheme_at_start_falls_back_to_default() {
        assert_eq!(engine("nope").scheme_id(), "default");
    }

    #[test]
    fn choosing_warm_sheen_lights_every_key_faintly() {
        let mut engine = engine("default");
        engine.set("warm", LightParams::default()).unwrap();
        assert_eq!(engine.scheme_id(), "warm");
        assert_eq!(engine.lights().unwrap().sets.len(), 1);
        assert_eq!(last_colour(&engine), Rgb::new(77, 64, 42));
    }

    #[test]
    fn changing_the_intensity_changes_the_colour() {
        let mut engine = engine("warm");
        let before = last_colour(&engine);
        engine.set("warm", LightParams { intensity: 60, ..LightParams::default() }).unwrap();
        assert!(last_colour(&engine).r > before.r);
    }

    #[test]
    fn settings_are_clamped() {
        let mut engine = engine("warm");
        engine.set("warm", LightParams { intensity: 200, speed: 99, ..LightParams::default() }).unwrap();
        assert_eq!((engine.params().intensity, engine.params().speed), (100, 10));
    }

    #[test]
    fn unknown_scheme_errors_and_keeps_the_current_one() {
        let mut engine = engine("warm");
        let sent = engine.lights().unwrap().sets.len();
        assert!(engine.set("nope", LightParams::default()).is_err());
        assert_eq!(engine.scheme_id(), "warm");
        assert_eq!(engine.lights().unwrap().sets.len(), sent);
    }

    #[test]
    fn static_schemes_send_nothing_on_tick() {
        let mut engine = engine("warm");
        let sent = engine.lights().unwrap().sets.len();
        engine.tick(Duration::from_secs(1));
        engine.tick(Duration::from_secs(2));
        assert_eq!(engine.lights().unwrap().sets.len(), sent);
        assert!(!engine.needs_ticks());
    }

    #[test]
    fn breathing_sends_a_new_colour_as_it_changes() {
        let mut engine = engine("breathing");
        assert!(engine.needs_ticks());
        let sent = engine.lights().unwrap().sets.len();
        for ms in [500, 1000, 1500, 2000] {
            engine.tick(Duration::from_millis(ms));
        }
        assert!(engine.lights().unwrap().sets.len() > sent);
    }

    #[test]
    fn an_unchanged_colour_is_not_sent_again() {
        let mut engine = engine("breathing");
        engine.tick(Duration::from_millis(700));
        let sent = engine.lights().unwrap().sets.len();
        engine.tick(Duration::from_millis(700));
        assert_eq!(engine.lights().unwrap().sets.len(), sent);
    }

    #[test]
    fn attaching_a_keyboard_applies_the_current_lighting() {
        let mut engine: LightEngine<MockLights> = LightEngine::new(None, "warm", LightParams::default());
        assert!(!engine.needs_ticks());
        engine.set_device(Some(MockLights::default()));
        assert_eq!(engine.lights().unwrap().sets.len(), 1);
    }

    #[test]
    fn a_failed_write_drops_the_keyboard() {
        let engine = LightEngine::new(Some(FailingLights), "warm", LightParams::default());
        assert!(engine.lights().is_none());
    }

    #[test]
    fn off_lights_nothing() {
        let engine = engine("off");
        assert_eq!(last_colour(&engine), Rgb::BLACK);
    }
}
