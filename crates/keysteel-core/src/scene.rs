use crate::frame::Frame;
use crate::scenes::{boing::Boing, mario::Mario, off::Off};
use std::time::Duration;

/// Something that can draw itself onto the OLED.
pub trait Scene {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    /// False when the picture never changes, so no frame timer is needed.
    fn animated(&self) -> bool;
    /// Clears `frame` and draws the picture for `elapsed` since the scene
    /// started. The result depends only on `elapsed`.
    fn render(&mut self, elapsed: Duration, frame: &mut Frame);
}

pub fn all_scenes() -> Vec<Box<dyn Scene>> {
    vec![Box::new(Off), Box::new(Boing::new()), Box::new(Mario::new())]
}

pub fn scene_by_id(id: &str) -> Option<Box<dyn Scene>> {
    all_scenes().into_iter().find(|scene| scene.id() == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_scenes_ids_are_off_boing_mario() {
        let ids: Vec<&str> = all_scenes().iter().map(|s| s.id()).collect();
        assert_eq!(ids, vec!["off", "boing", "mario"]);
    }

    #[test]
    fn scene_by_id_finds_known_scene() {
        assert_eq!(scene_by_id("boing").unwrap().id(), "boing");
    }

    #[test]
    fn scene_by_id_unknown_is_none() {
        assert!(scene_by_id("nope").is_none());
    }

    #[test]
    fn only_off_is_not_animated() {
        let flags: Vec<bool> = all_scenes().iter().map(|s| s.animated()).collect();
        assert_eq!(flags, vec![false, true, true]);
    }
}
