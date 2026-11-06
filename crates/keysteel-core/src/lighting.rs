// SPDX-License-Identifier: GPL-3.0-or-later
//
// Copyright © 2026 Adeptum AB and Adam Waldenberg
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

//! Key lighting: colours, the settings that shape them and the presets
//! ("schemes") the daemon can apply.

use std::f64::consts::TAU;
use std::time::Duration;

/// A colour, one byte per channel.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const BLACK: Rgb = Rgb { r: 0, g: 0, b: 0 };
    pub const WHITE: Rgb = Rgb { r: 255, g: 255, b: 255 };

    pub const fn new(r: u8, g: u8, b: u8) -> Rgb {
        Rgb { r, g, b }
    }

    /// The colour at `factor` of its strength; 0 is black, 1 is unchanged.
    pub fn scaled(self, factor: f64) -> Rgb {
        let factor = factor.clamp(0.0, 1.0);
        let channel = |value: u8| (value as f64 * factor).round() as u8;
        Rgb { r: channel(self.r), g: channel(self.g), b: channel(self.b) }
    }
}

/// HID usage codes of the keys of the Apex Pro TKL, ISO layout included.
pub const KEYS: [u8; 86] = [
    0x29, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f, 0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x35, 0x1e,
    0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x2d, 0x2e, 0x2a, 0x49, 0x4a, 0x4b,
    0x2b, 0x14, 0x1a, 0x08, 0x15, 0x17, 0x1c, 0x18, 0x0c, 0x12, 0x13, 0x2f, 0x30, 0x31, 0x4c,
    0x4d, 0x4e, 0x39, 0x04, 0x16, 0x07, 0x09, 0x0a, 0x0b, 0x0d, 0x0e, 0x0f, 0x33, 0x34, 0x28,
    0xe1, 0x1d, 0x1b, 0x06, 0x19, 0x05, 0x11, 0x10, 0x36, 0x37, 0x38, 0xe5, 0x52, 0xe0, 0xe3,
    0xe2, 0x2c, 0xe6, 0xe7, 0xf0, 0xe4, 0x50, 0x51, 0x4f, 0x32, 0x64,
];

pub const MAX_INTENSITY: u8 = 100;
pub const MIN_SPEED: u8 = 1;
pub const MAX_SPEED: u8 = 10;

/// What the user can tune, whichever scheme is chosen.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LightParams {
    /// Percent of full strength, 0 to 100.
    pub intensity: u8,
    /// The colour for schemes that take one.
    pub colour: Rgb,
    /// How quick the breathing is, 1 to 10.
    pub speed: u8,
}

impl Default for LightParams {
    fn default() -> LightParams {
        LightParams { intensity: 30, colour: Rgb::WHITE, speed: 4 }
    }
}

impl LightParams {
    /// The same settings with every value brought into range.
    pub fn clamped(self) -> LightParams {
        LightParams {
            intensity: self.intensity.min(MAX_INTENSITY),
            colour: self.colour,
            speed: self.speed.clamp(MIN_SPEED, MAX_SPEED),
        }
    }

    fn strength(&self) -> f64 {
        self.intensity.min(MAX_INTENSITY) as f64 / MAX_INTENSITY as f64
    }
}

/// What the keys should show.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lighting {
    /// Leave the lighting to the keyboard's own scheme.
    HandBack,
    /// Every key in this colour.
    All(Rgb),
}

/// A lighting preset.
pub trait LightScheme: Send {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    /// False when the result never changes, so no timer is needed.
    fn animated(&self) -> bool {
        false
    }
    /// True when the colour chosen by the user is used.
    fn takes_colour(&self) -> bool {
        false
    }
    /// True when the speed setting is used.
    fn uses_speed(&self) -> bool {
        false
    }
    /// True when the intensity setting is used.
    fn uses_intensity(&self) -> bool {
        true
    }
    /// What to show `elapsed` after the scheme was chosen. The result
    /// depends only on the arguments.
    fn render(&self, elapsed: Duration, params: &LightParams) -> Lighting;
}

/// A very faint, yellowish white.
pub struct WarmSheen;

/// The colour of the sheen at full strength.
const SHEEN: Rgb = Rgb::new(255, 214, 140);

impl LightScheme for WarmSheen {
    fn id(&self) -> &'static str {
        "warm"
    }
    fn name(&self) -> &'static str {
        "Warm sheen"
    }
    fn render(&self, _elapsed: Duration, params: &LightParams) -> Lighting {
        Lighting::All(SHEEN.scaled(params.strength()))
    }
}

/// One colour on every key.
pub struct Solid;

impl LightScheme for Solid {
    fn id(&self) -> &'static str {
        "solid"
    }
    fn name(&self) -> &'static str {
        "Solid colour"
    }
    fn takes_colour(&self) -> bool {
        true
    }
    fn render(&self, _elapsed: Duration, params: &LightParams) -> Lighting {
        Lighting::All(params.colour.scaled(params.strength()))
    }
}

/// One colour that slowly swells and fades.
pub struct Breathing;

/// How dark the breathing gets, as a share of the chosen intensity.
const BREATH_FLOOR: f64 = 0.06;

impl Breathing {
    /// Seconds for one breath at a speed from 1 to 10.
    fn period_secs(speed: u8) -> f64 {
        14.0 - speed.clamp(MIN_SPEED, MAX_SPEED) as f64 * 1.2
    }
}

impl LightScheme for Breathing {
    fn id(&self) -> &'static str {
        "breathing"
    }
    fn name(&self) -> &'static str {
        "Breathing"
    }
    fn animated(&self) -> bool {
        true
    }
    fn takes_colour(&self) -> bool {
        true
    }
    fn uses_speed(&self) -> bool {
        true
    }
    fn render(&self, elapsed: Duration, params: &LightParams) -> Lighting {
        // Only the position inside one breath matters; f64 keeps it exact
        // after days of uptime.
        let phase = (elapsed.as_secs_f64() / Breathing::period_secs(params.speed)).fract();
        let swell = 0.5 - 0.5 * (phase * TAU).cos();
        let level = BREATH_FLOOR + (1.0 - BREATH_FLOOR) * swell;
        Lighting::All(params.colour.scaled(params.strength() * level))
    }
}

/// All keys dark.
pub struct Dark;

impl LightScheme for Dark {
    fn id(&self) -> &'static str {
        "off"
    }
    fn name(&self) -> &'static str {
        "Off"
    }
    fn uses_intensity(&self) -> bool {
        false
    }
    fn render(&self, _elapsed: Duration, _params: &LightParams) -> Lighting {
        Lighting::All(Rgb::BLACK)
    }
}

/// The keyboard's own lighting.
pub struct KeyboardDefault;

impl LightScheme for KeyboardDefault {
    fn id(&self) -> &'static str {
        "default"
    }
    fn name(&self) -> &'static str {
        "Keyboard default"
    }
    fn uses_intensity(&self) -> bool {
        false
    }
    fn render(&self, _elapsed: Duration, _params: &LightParams) -> Lighting {
        Lighting::HandBack
    }
}

/// Every scheme, in the order the app lists them.
pub fn all_light_schemes() -> Vec<Box<dyn LightScheme>> {
    vec![
        Box::new(WarmSheen),
        Box::new(Solid),
        Box::new(Breathing),
        Box::new(Dark),
        Box::new(KeyboardDefault),
    ]
}

pub fn light_scheme_by_id(id: &str) -> Option<Box<dyn LightScheme>> {
    all_light_schemes().into_iter().find(|scheme| scheme.id() == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(secs: f64) -> Duration {
        Duration::from_secs_f64(secs)
    }

    fn params(intensity: u8) -> LightParams {
        LightParams { intensity, ..LightParams::default() }
    }

    fn all(lighting: Lighting) -> Rgb {
        match lighting {
            Lighting::All(colour) => colour,
            Lighting::HandBack => panic!("expected a colour"),
        }
    }

    #[test]
    fn scaling_keeps_black_and_full_strength() {
        assert_eq!(Rgb::new(255, 214, 140).scaled(0.0), Rgb::BLACK);
        assert_eq!(Rgb::new(255, 214, 140).scaled(1.0), Rgb::new(255, 214, 140));
        assert_eq!(Rgb::new(200, 100, 50).scaled(0.5), Rgb::new(100, 50, 25));
    }

    #[test]
    fn scaling_is_kept_in_range() {
        assert_eq!(Rgb::new(10, 20, 30).scaled(5.0), Rgb::new(10, 20, 30));
        assert_eq!(Rgb::new(10, 20, 30).scaled(-1.0), Rgb::BLACK);
    }

    #[test]
    fn key_list_has_no_repeats_and_the_iso_keys() {
        let mut sorted = KEYS.to_vec();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), KEYS.len());
        assert!(KEYS.contains(&0x32) && KEYS.contains(&0x64));
    }

    #[test]
    fn warm_sheen_is_a_faint_yellowish_white() {
        let colour = all(WarmSheen.render(at(0.0), &params(30)));
        assert_eq!(colour, Rgb::new(77, 64, 42));
        assert!(colour.r > colour.g && colour.g > colour.b);
        assert!(colour.r < 100);
    }

    #[test]
    fn warm_sheen_follows_the_intensity() {
        assert_eq!(all(WarmSheen.render(at(0.0), &params(0))), Rgb::BLACK);
        assert_eq!(all(WarmSheen.render(at(0.0), &params(100))), SHEEN);
        let (low, high) = (
            all(WarmSheen.render(at(0.0), &params(20))),
            all(WarmSheen.render(at(0.0), &params(60))),
        );
        assert!(low.r < high.r && low.g < high.g && low.b < high.b);
    }

    #[test]
    fn warm_sheen_ignores_the_colour_setting() {
        let red = LightParams { colour: Rgb::new(255, 0, 0), ..params(30) };
        assert_eq!(WarmSheen.render(at(0.0), &red), WarmSheen.render(at(0.0), &params(30)));
    }

    #[test]
    fn solid_uses_the_colour_and_intensity() {
        let p = LightParams { intensity: 50, colour: Rgb::new(0, 200, 100), speed: 4 };
        assert_eq!(all(Solid.render(at(0.0), &p)), Rgb::new(0, 100, 50));
    }

    #[test]
    fn off_is_dark_and_default_hands_back() {
        assert_eq!(Dark.render(at(1.0), &params(100)), Lighting::All(Rgb::BLACK));
        assert_eq!(KeyboardDefault.render(at(1.0), &params(100)), Lighting::HandBack);
    }

    #[test]
    fn breathing_swells_and_fades_within_the_intensity() {
        let p = LightParams { intensity: 80, colour: Rgb::WHITE, speed: 5 };
        let period = Breathing::period_secs(5);
        let dark = all(Breathing.render(at(0.0), &p));
        let bright = all(Breathing.render(at(period / 2.0), &p));
        assert!(dark.r < 20);
        assert_eq!(bright.r, 204);
        for step in 0..200 {
            let colour = all(Breathing.render(at(step as f64 * period / 200.0), &p));
            assert!(colour.r <= 204);
        }
    }

    #[test]
    fn breathing_speed_shortens_the_breath() {
        assert!(Breathing::period_secs(10) < Breathing::period_secs(1));
        assert!(Breathing::period_secs(1) < 14.0 && Breathing::period_secs(10) >= 2.0);
    }

    #[test]
    fn breathing_repeats_each_breath() {
        let p = LightParams { speed: 5, ..params(60) };
        let period = Breathing::period_secs(5);
        assert_eq!(Breathing.render(at(1.5), &p), Breathing.render(at(1.5 + period * 4.0), &p));
    }

    #[test]
    fn breathing_still_moves_after_30_days_of_uptime() {
        let p = params(60);
        let base = 30.0 * 24.0 * 3600.0;
        assert_ne!(Breathing.render(at(base), &p), Breathing.render(at(base + 1.0), &p));
    }

    #[test]
    fn params_are_clamped() {
        let wild = LightParams { intensity: 250, colour: Rgb::WHITE, speed: 99 }.clamped();
        assert_eq!((wild.intensity, wild.speed), (100, 10));
        assert_eq!(LightParams { speed: 0, ..params(5) }.clamped().speed, 1);
    }

    #[test]
    fn schemes_are_listed_in_menu_order_and_found_by_id() {
        let ids: Vec<&str> = all_light_schemes().iter().map(|s| s.id()).collect();
        assert_eq!(ids, vec!["warm", "solid", "breathing", "off", "default"]);
        assert_eq!(light_scheme_by_id("solid").unwrap().id(), "solid");
        assert!(light_scheme_by_id("nope").is_none());
    }

    #[test]
    fn only_breathing_is_animated() {
        let flags: Vec<bool> = all_light_schemes().iter().map(|s| s.animated()).collect();
        assert_eq!(flags, vec![false, false, true, false, false]);
    }
}
