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
}
