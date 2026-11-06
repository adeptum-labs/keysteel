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

use keysteel_core::lighting::{light_scheme_by_id, LightParams, Rgb};
use keysteel_core::scene::scene_by_id;
use std::path::{Path, PathBuf};

const DEFAULT_SCENE: &str = "off";

/// `$XDG_STATE_HOME/keysteel`, falling back to `~/.local/state/keysteel`.
fn state_dir() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/state")))?;
    Some(base.join("keysteel"))
}

/// Where the chosen scene is kept.
pub fn default_path() -> Option<PathBuf> {
    state_dir().map(|dir| dir.join("scene"))
}

/// Where the lighting setting is kept.
pub fn lighting_path() -> Option<PathBuf> {
    state_dir().map(|dir| dir.join("lighting"))
}

const DEFAULT_LIGHT_SCHEME: &str = "default";

/// Reads the saved lighting as scheme id and settings. A missing, unreadable
/// or unknown value gives the keyboard's own lighting with default settings.
pub fn load_lighting(path: &Path) -> (String, LightParams) {
    let fallback = (DEFAULT_LIGHT_SCHEME.to_string(), LightParams::default());
    let Ok(text) = std::fs::read_to_string(path) else {
        return fallback;
    };
    let mut fields = text.split_whitespace();
    let (Some(id), Some(intensity), Some(colour), Some(speed)) =
        (fields.next(), fields.next(), fields.next(), fields.next())
    else {
        return fallback;
    };
    let parsed = (
        intensity.parse::<u8>(),
        u32::from_str_radix(colour, 16),
        speed.parse::<u8>(),
    );
    match parsed {
        (Ok(intensity), Ok(rgb), Ok(speed)) if colour.len() == 6 && light_scheme_by_id(id).is_some() => (
            id.to_string(),
            LightParams {
                intensity,
                colour: Rgb::new((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8),
                speed,
            }
            .clamped(),
        ),
        _ => fallback,
    }
}

pub fn save_lighting(path: &Path, id: &str, params: &LightParams) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let c = params.colour;
    std::fs::write(path, format!("{id} {} {:02x}{:02x}{:02x} {}", params.intensity, c.r, c.g, c.b, params.speed))
}

/// Where the roller setting is kept.
pub fn roller_path() -> Option<PathBuf> {
    state_dir().map(|dir| dir.join("roller"))
}

pub const MIN_SPEED: u32 = 1;
pub const MAX_SPEED: u32 = 10;
pub const DEFAULT_SPEED: u32 = 3;

/// Keeps a roller speed (wheel notches per click) in the allowed range.
pub fn clamp_speed(speed: u32) -> u32 {
    speed.clamp(MIN_SPEED, MAX_SPEED)
}

/// Where the roller speed is kept.
pub fn speed_path() -> Option<PathBuf> {
    state_dir().map(|dir| dir.join("roller-speed"))
}

/// Reads the roller speed; a missing or unreadable value gives the default.
pub fn load_speed(path: &Path) -> u32 {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| text.trim().parse().ok())
        .map(clamp_speed)
        .unwrap_or(DEFAULT_SPEED)
}

pub fn save_speed(path: &Path, speed: u32) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, speed.to_string())
}

/// Reads the roller setting; a missing or unreadable file means off.
pub fn load_roller(path: &Path) -> bool {
    std::fs::read_to_string(path).map(|text| text.trim() == "on").unwrap_or(false)
}

pub fn save_roller(path: &Path, on: bool) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, if on { "on" } else { "off" })
}

/// Reads the saved scene id; a missing, unreadable or unknown value gives Off.
pub fn load_from(path: &Path) -> String {
    std::fs::read_to_string(path)
        .map(|text| text.trim().to_string())
        .ok()
        .filter(|id| scene_by_id(id).is_some())
        .unwrap_or_else(|| DEFAULT_SCENE.to_string())
}

pub fn save_to(path: &Path, id: &str) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(path, id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_path(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("keysteel-test-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir.join("scene")
    }

    #[test]
    fn state_missing_file_is_off() {
        assert_eq!(load_from(&temp_path("missing")), "off");
    }

    #[test]
    fn state_corrupt_file_is_off() {
        let path = temp_path("corrupt");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, [0xff, 0xfe, 0x00]).unwrap();
        assert_eq!(load_from(&path), "off");
    }

    #[test]
    fn state_unknown_scene_is_off() {
        let path = temp_path("unknown");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "garbage").unwrap();
        assert_eq!(load_from(&path), "off");
    }

    #[test]
    fn state_roundtrip() {
        let path = temp_path("roundtrip");
        save_to(&path, "mario").unwrap();
        assert_eq!(load_from(&path), "mario");
    }

    #[test]
    fn roller_defaults_to_off() {
        assert!(!load_roller(&temp_path("roller-missing")));
    }

    #[test]
    fn roller_setting_roundtrips() {
        let path = temp_path("roller");
        save_roller(&path, true).unwrap();
        assert!(load_roller(&path));
        save_roller(&path, false).unwrap();
        assert!(!load_roller(&path));
    }

    #[test]
    fn speed_defaults_to_three() {
        assert_eq!(load_speed(&temp_path("speed-missing")), 3);
    }

    #[test]
    fn speed_roundtrips() {
        let path = temp_path("speed");
        save_speed(&path, 7).unwrap();
        assert_eq!(load_speed(&path), 7);
    }

    #[test]
    fn speed_is_kept_in_range() {
        assert_eq!(clamp_speed(0), 1);
        assert_eq!(clamp_speed(99), 10);
        assert_eq!(clamp_speed(5), 5);
        let path = temp_path("speed-big");
        save_speed(&path, 500).unwrap();
        assert_eq!(load_speed(&path), 10);
    }

    #[test]
    fn speed_garbage_is_the_default() {
        let path = temp_path("speed-garbage");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "fast").unwrap();
        assert_eq!(load_speed(&path), 3);
    }

    #[test]
    fn lighting_defaults_to_the_keyboards_own() {
        let (id, params) = load_lighting(&temp_path("light-missing"));
        assert_eq!(id, "default");
        assert_eq!(params, LightParams::default());
    }

    #[test]
    fn lighting_roundtrips() {
        let path = temp_path("light");
        let params = LightParams { intensity: 42, colour: Rgb::new(255, 214, 140), speed: 7 };
        save_lighting(&path, "breathing", &params).unwrap();
        assert_eq!(load_lighting(&path), ("breathing".to_string(), params));
    }

    #[test]
    fn lighting_garbage_gives_the_default() {
        for text in ["", "warm", "nope 30 ffffff 4", "warm x ffffff 4", "warm 30 zzzzzz 4", "warm 30 fff 4"] {
            let path = temp_path("light-garbage");
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, text).unwrap();
            assert_eq!(load_lighting(&path).0, "default", "for {text:?}");
        }
    }

    #[test]
    fn lighting_out_of_range_values_are_clamped() {
        let path = temp_path("light-range");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "warm 250 ffffff 99").unwrap();
        let (_, params) = load_lighting(&path);
        assert_eq!((params.intensity, params.speed), (100, 10));
    }
}
