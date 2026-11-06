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

//! Turns the keyboard's roller into mouse wheel scrolling.
//!
//! The roller sends volume keys. While the feature is on, the daemon takes
//! the keyboard's media key device for itself, passes every key through
//! except the volume keys, and turns those into wheel events on a virtual
//! mouse.

use evdev::{EventType, KeyCode};

/// What to do with one input event from the media key device.
#[derive(Debug, PartialEq, Eq)]
pub enum Mapped {
    /// Forward it unchanged.
    Pass,
    /// Scroll this many notches; positive is up, away from the user.
    Wheel(i32),
    /// Drop it.
    Swallow,
}

/// Decides what happens to the event with this type, code and value.
pub fn map(kind: u16, code: u16, value: i32) -> Mapped {
    if kind != EventType::KEY.0 {
        return Mapped::Pass;
    }
    let up = code == KeyCode::KEY_VOLUMEUP.code();
    let down = code == KeyCode::KEY_VOLUMEDOWN.code();
    match (up || down, value) {
        (false, _) => Mapped::Pass,
        // Release of a volume key: nothing to scroll.
        (true, 0) => Mapped::Swallow,
        // Press, or the key repeating.
        (true, _) => Mapped::Wheel(if up { 1 } else { -1 }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: u16 = 1;
    const SYN: u16 = 0;

    #[test]
    fn volume_up_scrolls_up() {
        assert_eq!(map(KEY, KeyCode::KEY_VOLUMEUP.code(), 1), Mapped::Wheel(1));
    }

    #[test]
    fn volume_down_scrolls_down() {
        assert_eq!(map(KEY, KeyCode::KEY_VOLUMEDOWN.code(), 1), Mapped::Wheel(-1));
    }

    #[test]
    fn a_held_roller_key_keeps_scrolling() {
        assert_eq!(map(KEY, KeyCode::KEY_VOLUMEUP.code(), 2), Mapped::Wheel(1));
    }

    #[test]
    fn releasing_a_volume_key_does_nothing() {
        assert_eq!(map(KEY, KeyCode::KEY_VOLUMEUP.code(), 0), Mapped::Swallow);
        assert_eq!(map(KEY, KeyCode::KEY_VOLUMEDOWN.code(), 0), Mapped::Swallow);
    }

    #[test]
    fn other_keys_pass_through() {
        assert_eq!(map(KEY, KeyCode::KEY_MUTE.code(), 1), Mapped::Pass);
        assert_eq!(map(KEY, KeyCode::KEY_PLAYPAUSE.code(), 0), Mapped::Pass);
    }

    #[test]
    fn non_key_events_pass_through() {
        assert_eq!(map(SYN, 0, 0), Mapped::Pass);
    }
}
