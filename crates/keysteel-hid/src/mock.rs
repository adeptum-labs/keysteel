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

use crate::{Device, DeviceError, KeyColour, KeyLights};
use keysteel_core::frame::Frame;

/// Records every frame it is sent. Used in tests.
#[derive(Default)]
pub struct MockDevice {
    pub frames: Vec<Frame>,
    pub resets: usize,
}

impl Device for MockDevice {
    fn send_frame(&mut self, frame: &Frame) -> Result<(), DeviceError> {
        self.frames.push(frame.clone());
        Ok(())
    }

    fn reset(&mut self) -> Result<(), DeviceError> {
        self.resets += 1;
        Ok(())
    }
}

/// Records every lighting change. Used in tests.
#[derive(Default)]
pub struct MockLights {
    pub sets: Vec<Vec<KeyColour>>,
    pub clears: usize,
}

impl KeyLights for MockLights {
    fn set_keys(&mut self, keys: &[KeyColour]) -> Result<(), DeviceError> {
        self.sets.push(keys.to_vec());
        Ok(())
    }

    fn clear_keys(&mut self) -> Result<(), DeviceError> {
        self.clears += 1;
        Ok(())
    }
}
