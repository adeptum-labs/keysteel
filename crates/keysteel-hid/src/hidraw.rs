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

use crate::{
    encode_clear_lights_report, encode_lights_report, encode_report, encode_reset_report, Device, DeviceError,
    KeyColour, KeyLights,
};
use keysteel_core::frame::Frame;
use std::sync::Mutex;
use std::time::Duration;

pub const VENDOR_ID: u16 = 0x1038;
pub const PRODUCT_ID: u16 = 0x1642;
/// The OLED is driven through this HID interface.
pub const INTERFACE: i32 = 1;

/// True for the kernel's `HID_ID` value (`bus:vendor:product`, each in hex)
/// of the Apex Pro TKL Gen 3.
pub fn is_keyboard_hid_id(hid_id: &str) -> bool {
    let mut parts = hid_id.split(':').skip(1).map(|p| u32::from_str_radix(p, 16));
    matches!(
        (parts.next(), parts.next(), parts.next()),
        (Some(Ok(vendor)), Some(Ok(product)), None)
            if vendor == VENDOR_ID as u32 && product == PRODUCT_ID as u32
    )
}

/// The keyboard takes one report at a time. The display and the lighting use
/// separate handles from separate threads, and reports that overlap can make
/// the keyboard refuse one with a "Broken pipe".
static BUS: Mutex<()> = Mutex::new(());

const SEND_ATTEMPTS: u32 = 3;
const SEND_RETRY: Duration = Duration::from_millis(10);

/// Runs `send` up to `attempts` times, pausing `delay` after each failure.
/// A refused report is usually accepted when sent again a moment later.
fn retried(attempts: u32, delay: Duration, mut send: impl FnMut() -> Result<(), String>) -> Result<(), DeviceError> {
    let mut last = String::new();
    for _ in 0..attempts.max(1) {
        match send() {
            Ok(()) => return Ok(()),
            Err(message) => {
                last = message;
                std::thread::sleep(delay);
            }
        }
    }
    Err(DeviceError::Hid(last))
}

fn send_feature(handle: &hidapi::HidDevice, report: &[u8]) -> Result<(), DeviceError> {
    let _turn = BUS.lock().unwrap_or_else(|e| e.into_inner());
    retried(SEND_ATTEMPTS, SEND_RETRY, || handle.send_feature_report(report).map_err(|e| e.to_string()))
}

fn send_output(handle: &hidapi::HidDevice, report: &[u8]) -> Result<(), DeviceError> {
    let _turn = BUS.lock().unwrap_or_else(|e| e.into_inner());
    retried(SEND_ATTEMPTS, SEND_RETRY, || handle.write(report).map(|_| ()).map_err(|e| e.to_string()))
}

/// Opens the interface that takes the display and lighting reports.
fn open_handle() -> Result<hidapi::HidDevice, DeviceError> {
    let api = hidapi::HidApi::new().map_err(|e| DeviceError::Hid(e.to_string()))?;
    let info = api
        .device_list()
        .find(|d| {
            d.vendor_id() == VENDOR_ID && d.product_id() == PRODUCT_ID && d.interface_number() == INTERFACE
        })
        .ok_or(DeviceError::NotFound)?;
    info.open_device(&api).map_err(|e| DeviceError::Hid(e.to_string()))
}

pub struct HidrawDevice {
    handle: hidapi::HidDevice,
}

impl HidrawDevice {
    /// Opens the display interface of the Apex Pro TKL Gen 3.
    pub fn open() -> Result<HidrawDevice, DeviceError> {
        Ok(HidrawDevice { handle: open_handle()? })
    }
}

/// The key lighting of the Apex Pro TKL Gen 3, on its own handle so it works
/// independently of the display.
pub struct HidrawLights {
    handle: hidapi::HidDevice,
}

impl HidrawLights {
    pub fn open() -> Result<HidrawLights, DeviceError> {
        Ok(HidrawLights { handle: open_handle()? })
    }
}

impl KeyLights for HidrawLights {
    fn set_keys(&mut self, keys: &[KeyColour]) -> Result<(), DeviceError> {
        send_feature(&self.handle, &encode_lights_report(keys))
    }

    fn clear_keys(&mut self) -> Result<(), DeviceError> {
        send_output(&self.handle, &encode_clear_lights_report())
    }
}

impl Device for HidrawDevice {
    fn send_frame(&mut self, frame: &Frame) -> Result<(), DeviceError> {
        send_feature(&self.handle, &encode_report(frame))
    }

    fn reset(&mut self) -> Result<(), DeviceError> {
        send_output(&self.handle, &encode_reset_report())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hid_id_of_the_keyboard_matches() {
        assert!(is_keyboard_hid_id("0003:00001038:00001642"));
    }

    #[test]
    fn hid_id_of_another_product_does_not_match() {
        assert!(!is_keyboard_hid_id("0003:00001038:00001618"));
        assert!(!is_keyboard_hid_id("0003:0000046D:0000C52B"));
    }

    #[test]
    fn garbage_hid_id_does_not_match() {
        assert!(!is_keyboard_hid_id(""));
        assert!(!is_keyboard_hid_id("1038:1642"));
        assert!(!is_keyboard_hid_id("0003:zzzz:yyyy"));
    }

    #[test]
    fn hid_id_of_another_vendor_with_the_same_product_does_not_match() {
        assert!(!is_keyboard_hid_id("0003:0000103a:00001642"));
    }

    #[test]
    fn a_refused_report_is_sent_again() {
        let mut calls = 0;
        let result = retried(3, Duration::ZERO, || {
            calls += 1;
            if calls < 3 {
                Err("Broken pipe".to_string())
            } else {
                Ok(())
            }
        });
        assert!(result.is_ok());
        assert_eq!(calls, 3);
    }

    #[test]
    fn a_report_that_keeps_failing_gives_up_with_the_reason() {
        let mut calls = 0;
        let result = retried(3, Duration::ZERO, || {
            calls += 1;
            Err("No such device".to_string())
        });
        assert_eq!(calls, 3);
        assert!(matches!(result, Err(DeviceError::Hid(ref m)) if m == "No such device"));
    }

    #[test]
    fn a_report_that_goes_through_is_sent_once() {
        let mut calls = 0;
        retried(3, Duration::ZERO, || {
            calls += 1;
            Ok(())
        })
        .unwrap();
        assert_eq!(calls, 1);
    }
}
