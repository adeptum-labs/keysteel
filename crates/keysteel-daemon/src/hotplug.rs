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

use crate::light_runtime::LightShared;
use crate::runtime::Shared;
use keysteel_hid::{is_keyboard_hid_id, DeviceError, HidrawDevice, HidrawLights};
use std::os::unix::io::AsRawFd;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

/// Device nodes can take a moment to become accessible after they appear.
const OPEN_ATTEMPTS: u32 = 10;
const OPEN_RETRY: Duration = Duration::from_millis(200);

/// Runs `open` until it succeeds, up to `attempts` times with `delay` between
/// them, and says why when it never does.
fn open_with_retries<T>(
    what: &str,
    attempts: u32,
    delay: Duration,
    mut open: impl FnMut() -> Result<T, DeviceError>,
) -> Option<T> {
    let mut last_error = None;
    for _ in 0..attempts {
        match open() {
            Ok(device) => return Some(device),
            Err(e) => {
                last_error = Some(e);
                sleep(delay);
            }
        }
    }
    if let Some(e) = last_error {
        eprintln!("cannot open the keyboard {what}: {e}");
    }
    None
}

/// Opens the keyboard and gives it to the display and lighting engines,
/// replacing any handle they already hold. A handle can go stale without a
/// failed send (unplugged while nothing was being sent), so a node appearing
/// always wins. The two are opened separately, so one failing does not leave
/// the other without a keyboard.
pub fn try_attach(shared: &Shared, lights: &LightShared) {
    if let Some(device) = open_with_retries("display", OPEN_ATTEMPTS, OPEN_RETRY, HidrawDevice::open) {
        shared.lock().engine.set_device(Some(device));
        shared.wake();
    }
    if let Some(handle) = open_with_retries("lighting", OPEN_ATTEMPTS, OPEN_RETRY, HidrawLights::open) {
        lights.lock().engine.set_device(Some(handle));
        lights.wake();
    }
}

/// True when a hidraw node of the keyboard itself appeared, not one of some
/// other device. The kernel names the device in the `HID_ID` of its parent.
fn is_keyboard_added(event: &udev::Event) -> bool {
    event.event_type() == udev::EventType::Add
        && event
            .parent_with_subsystem("hid")
            .ok()
            .flatten()
            .and_then(|hid| hid.property_value("HID_ID").and_then(|v| v.to_str()).map(is_keyboard_hid_id))
            .unwrap_or(false)
}

/// Blocks forever, attaching the keyboard whenever a hidraw node appears.
pub fn watch(shared: Arc<Shared>, lights: Arc<LightShared>) -> std::io::Result<()> {
    let monitor = udev::MonitorBuilder::new()?.match_subsystem("hidraw")?.listen()?;
    let mut poll_fd = libc::pollfd { fd: monitor.as_raw_fd(), events: libc::POLLIN, revents: 0 };
    loop {
        // SAFETY: `poll_fd` is a valid pollfd and the count is 1.
        let ready = unsafe { libc::poll(&mut poll_fd, 1, -1) };
        if ready < 0 {
            let error = std::io::Error::last_os_error();
            if error.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            return Err(error);
        }
        let added = monitor.iter().any(|event| is_keyboard_added(&event));
        if added {
            try_attach(&shared, &lights);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opening_is_retried_until_it_works() {
        let mut calls = 0;
        let opened = open_with_retries("test", 5, Duration::ZERO, || {
            calls += 1;
            if calls < 3 {
                Err(DeviceError::NotFound)
            } else {
                Ok(calls)
            }
        });
        assert_eq!(opened, Some(3));
    }

    #[test]
    fn opening_gives_up_after_the_attempts() {
        let mut calls = 0;
        let opened: Option<u32> = open_with_retries("test", 4, Duration::ZERO, || {
            calls += 1;
            Err(DeviceError::Hid("busy".into()))
        });
        assert_eq!(opened, None);
        assert_eq!(calls, 4);
    }

    #[test]
    fn opening_that_works_at_once_is_not_repeated() {
        let mut calls = 0;
        let opened = open_with_retries("test", 4, Duration::ZERO, || {
            calls += 1;
            Ok::<_, DeviceError>(7)
        });
        assert_eq!((opened, calls), (Some(7), 1));
    }
}
