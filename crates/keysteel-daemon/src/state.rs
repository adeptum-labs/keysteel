use keysteel_core::scene::scene_by_id;
use std::path::{Path, PathBuf};

const DEFAULT_SCENE: &str = "off";

/// `$XDG_STATE_HOME/keysteel/scene`, falling back to `~/.local/state`.
pub fn default_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/state")))?;
    Some(base.join("keysteel").join("scene"))
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
}
