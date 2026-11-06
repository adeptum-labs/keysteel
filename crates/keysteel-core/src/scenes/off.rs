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

use crate::frame::Frame;
use crate::scene::Scene;
use std::time::Duration;

/// Shows nothing.
pub struct Off;

impl Scene for Off {
    fn id(&self) -> &'static str {
        "off"
    }

    fn name(&self) -> &'static str {
        "Off"
    }

    fn animated(&self) -> bool {
        false
    }

    fn shows_picture(&self) -> bool {
        false
    }

    fn render(&mut self, _elapsed: Duration, frame: &mut Frame) {
        frame.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::Scene;
    use std::time::Duration;

    #[test]
    fn off_renders_blank() {
        let mut frame = Frame::new();
        frame.set_pixel(3, 3, true);
        Off.render(Duration::from_secs(1), &mut frame);
        assert_eq!(frame, Frame::new());
    }
}
