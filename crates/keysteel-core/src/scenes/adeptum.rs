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

use crate::frame::{Frame, HEIGHT, WIDTH};
use crate::scene::Scene;
use crate::scenes::sprite::{parse_sprite_frames, SpriteFrame};
use std::time::Duration;

const PIXELS_PER_SEC: f64 = 40.0;

/// The Adeptum logo scrolling across the display, right to left.
pub struct Adeptum {
    logo: SpriteFrame,
}

impl Adeptum {
    pub fn new() -> Adeptum {
        let logo = parse_sprite_frames(include_str!("../../assets/adeptum.txt"))
            .into_iter()
            .next()
            .expect("the logo asset holds one picture");
        Adeptum { logo }
    }
}

impl Default for Adeptum {
    fn default() -> Adeptum {
        Adeptum::new()
    }
}

impl Scene for Adeptum {
    fn id(&self) -> &'static str {
        "adeptum"
    }

    fn name(&self) -> &'static str {
        "Adeptum logo"
    }

    fn animated(&self) -> bool {
        true
    }

    fn render(&mut self, elapsed: Duration, frame: &mut Frame) {
        frame.clear();
        // From just off the right edge to just off the left one, then again.
        // f64 keeps the position exact after days of uptime.
        let lap = (WIDTH + self.logo.width) as u64;
        let travelled = (elapsed.as_secs_f64() * PIXELS_PER_SEC) as u64 % lap;
        let left = WIDTH as i32 - travelled as i32;
        let top = (HEIGHT as i32 - self.logo.height as i32) / 2;
        for y in 0..self.logo.height {
            for x in 0..self.logo.width {
                if self.logo.get(x, y) {
                    frame.set_pixel(left + x as i32, top + y as i32, true);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render_at(ms: u64) -> Frame {
        let mut frame = Frame::new();
        Adeptum::new().render(Duration::from_millis(ms), &mut frame);
        frame
    }

    fn lit(frame: &Frame) -> usize {
        (0..HEIGHT)
            .flat_map(|y| (0..WIDTH).map(move |x| (x, y)))
            .filter(|&(x, y)| frame.get_pixel(x, y))
            .count()
    }

    #[test]
    fn the_logo_fits_the_screen_height() {
        assert!(Adeptum::new().logo.height <= HEIGHT);
    }

    #[test]
    fn the_logo_starts_off_screen_and_enters_from_the_right() {
        assert_eq!(lit(&render_at(0)), 0);
        assert!(lit(&render_at(500)) > 0);
    }

    #[test]
    fn the_logo_moves() {
        assert_ne!(render_at(1000), render_at(1100));
    }

    #[test]
    fn the_logo_is_wider_than_the_screen_and_scrolls_through_it() {
        // Too wide to show at once: the icon leaves before the text arrives.
        let total = Adeptum::new().logo.pixels.iter().filter(|p| **p).count();
        assert!(Adeptum::new().logo.width > WIDTH);
        assert!((0..8000).step_by(5).all(|ms| lit(&render_at(ms)) < total));
    }

    #[test]
    fn the_scroll_repeats() {
        // The logo moves 40 pixels a second, so a pixel takes 25 ms.
        let lap_ms = (WIDTH + Adeptum::new().logo.width) as u64 * 25;
        assert_eq!(render_at(2000), render_at(2000 + lap_ms * 40));
    }

    #[test]
    fn the_logo_still_moves_after_30_days_of_uptime() {
        let thirty_days = 30 * 24 * 3600 * 1000;
        assert_ne!(render_at(thirty_days + 1000), render_at(thirty_days + 1100));
    }
}
