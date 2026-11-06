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

mod hidraw;
mod mock;
mod report;

pub use hidraw::{is_keyboard_hid_id, HidrawDevice, HidrawLights};
pub use mock::{MockDevice, MockLights};
pub use report::{
    encode_clear_lights_report, encode_lights_report, encode_report, encode_reset_report, MAX_LIT_KEYS,
    REPORT_LEN,
};

use keysteel_core::frame::Frame;
use keysteel_core::lighting::{Rgb, KEYS};

#[derive(Debug)]
pub enum DeviceError {
    NotFound,
    Hid(String),
}

impl std::fmt::Display for DeviceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceError::NotFound => write!(f, "keyboard not found"),
            DeviceError::Hid(message) => write!(f, "hid error: {message}"),
        }
    }
}

impl std::error::Error for DeviceError {}

/// A display that can show one frame at a time.
pub trait Device {
    fn send_frame(&mut self, frame: &Frame) -> Result<(), DeviceError>;
    /// Gives the screen back to the keyboard's own display.
    fn reset(&mut self) -> Result<(), DeviceError>;
}

/// One key and the colour it should show.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct KeyColour {
    /// HID usage code of the key.
    pub key: u8,
    pub colour: Rgb,
}

/// The key lighting of a keyboard.
pub trait KeyLights {
    /// Sets the colour of each listed key; the rest are left as they are.
    fn set_keys(&mut self, keys: &[KeyColour]) -> Result<(), DeviceError>;

    /// Gives the key lighting back to the keyboard's own scheme.
    fn clear_keys(&mut self) -> Result<(), DeviceError>;

    /// Lights every key in one colour.
    fn set_all(&mut self, colour: Rgb) -> Result<(), DeviceError> {
        let keys: Vec<KeyColour> = KEYS.iter().map(|&key| KeyColour { key, colour }).collect();
        self.set_keys(&keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_records_frames() {
        let mut device = MockDevice::default();
        let mut frame = Frame::new();
        frame.set_pixel(1, 2, true);
        device.send_frame(&frame).unwrap();
        device.send_frame(&Frame::new()).unwrap();
        assert_eq!(device.frames.len(), 2);
        assert_eq!(device.frames[0], frame);
    }

    #[test]
    fn mock_counts_resets() {
        let mut device = MockDevice::default();
        device.reset().unwrap();
        device.reset().unwrap();
        assert_eq!(device.resets, 2);
        assert!(device.frames.is_empty());
    }

    #[test]
    fn mock_lights_record_colours_and_clears() {
        let mut lights = MockLights::default();
        lights.set_all(Rgb::new(9, 8, 7)).unwrap();
        lights.clear_keys().unwrap();
        assert_eq!(lights.sets.len(), 1);
        assert_eq!(lights.sets[0].len(), KEYS.len());
        assert!(lights.sets[0].iter().all(|k| k.colour == Rgb::new(9, 8, 7)));
        assert_eq!(lights.clears, 1);
    }
}
