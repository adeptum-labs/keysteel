// SPDX-License-Identifier: GPL-3.0-or-later
//
// Copyright © 2026 Adeptum AB, Org.nr 559494-1824, and Adam Waldenberg
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

use crate::frame::Frame;
use crate::scenes::{adeptum::Adeptum, boing::Boing, mario::Mario, off::Off};
use std::time::Duration;

/// Something that can draw itself onto the OLED.
pub trait Scene: Send {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    /// False when the picture never changes, so no frame timer is needed.
    fn animated(&self) -> bool;
    /// False for a scene that leaves the screen to the keyboard's own display.
    fn shows_picture(&self) -> bool {
        true
    }
    /// Clears `frame` and draws the picture for `elapsed` since the scene
    /// started. The result depends only on `elapsed`.
    fn render(&mut self, elapsed: Duration, frame: &mut Frame);
}

pub fn all_scenes() -> Vec<Box<dyn Scene>> {
    vec![Box::new(Off), Box::new(Boing::new()), Box::new(Mario::new()), Box::new(Adeptum::new())]
}

pub fn scene_by_id(id: &str) -> Option<Box<dyn Scene>> {
    all_scenes().into_iter().find(|scene| scene.id() == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_scenes_ids_are_in_menu_order() {
        let ids: Vec<&str> = all_scenes().iter().map(|s| s.id()).collect();
        assert_eq!(ids, vec!["off", "boing", "mario", "adeptum"]);
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
    fn only_off_hands_the_display_back() {
        let flags: Vec<bool> = all_scenes().iter().map(|s| s.shows_picture()).collect();
        assert_eq!(flags, vec![false, true, true, true]);
    }

    #[test]
    fn only_off_is_not_animated() {
        let flags: Vec<bool> = all_scenes().iter().map(|s| s.animated()).collect();
        assert_eq!(flags, vec![false, true, true, true]);
    }
}
