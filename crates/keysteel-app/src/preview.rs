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

use keysteel_core::frame::{Frame, HEIGHT, WIDTH};

/// Colour of a lit OLED pixel, as RGBA.
pub const ON: [u8; 4] = [0xe8, 0xe8, 0xe8, 0xff];
/// Colour of a dark OLED pixel, as RGBA.
pub const OFF: [u8; 4] = [0x10, 0x14, 0x1a, 0xff];

/// Width and height in pixels of a frame drawn at `scale` times its size.
pub fn preview_size(scale: usize) -> (usize, usize) {
    (WIDTH * scale, HEIGHT * scale)
}

/// Draws `frame` as RGBA bytes, each OLED pixel becoming a `scale` x `scale` block.
pub fn scale_frame_to_rgba(frame: &Frame, scale: usize) -> Vec<u8> {
    let (width, height) = preview_size(scale);
    let mut rgba = Vec::with_capacity(width * height * 4);
    for y in 0..height {
        for x in 0..width {
            let on = frame.get_pixel(x / scale, y / scale);
            rgba.extend_from_slice(if on { &ON } else { &OFF });
        }
    }
    rgba
}

#[cfg(test)]
mod tests {
    use super::*;
    use keysteel_core::frame::Frame;

    fn pixel(rgba: &[u8], width: usize, x: usize, y: usize) -> [u8; 4] {
        let i = (y * width + x) * 4;
        [rgba[i], rgba[i + 1], rgba[i + 2], rgba[i + 3]]
    }

    #[test]
    fn size_is_frame_size_times_scale() {
        assert_eq!(preview_size(3), (384, 120));
    }

    #[test]
    fn buffer_has_four_bytes_per_scaled_pixel() {
        assert_eq!(scale_frame_to_rgba(&Frame::new(), 2).len(), 256 * 80 * 4);
    }

    #[test]
    fn set_pixel_fills_its_whole_scaled_block() {
        let mut frame = Frame::new();
        frame.set_pixel(0, 0, true);
        let rgba = scale_frame_to_rgba(&frame, 2);
        for (x, y) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            assert_eq!(pixel(&rgba, 256, x, y), ON, "({x},{y})");
        }
        assert_eq!(pixel(&rgba, 256, 2, 0), OFF);
        assert_eq!(pixel(&rgba, 256, 0, 2), OFF);
    }

    #[test]
    fn last_pixel_lands_in_bottom_right_corner() {
        let mut frame = Frame::new();
        frame.set_pixel(127, 39, true);
        let rgba = scale_frame_to_rgba(&frame, 3);
        assert_eq!(pixel(&rgba, 384, 383, 119), ON);
        assert_eq!(pixel(&rgba, 384, 381, 117), ON);
        assert_eq!(pixel(&rgba, 384, 380, 117), OFF);
        assert_eq!(pixel(&rgba, 384, 381, 116), OFF);
    }
}
