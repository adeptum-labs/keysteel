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

//! Tries per-key lighting on the keyboard: `lightspike <r> <g> <b> [seconds]`
//! lights every key with that colour and then waits, `lightspike clear`
//! hands the lighting back to the keyboard.

use std::time::Duration;

/// HID usage codes of the keys of the Apex Pro TKL layout.
const KEYS: [u8; 87] = [
    0x29, 0x3a, 0x3b, 0x3c, 0x3d, 0x3e, 0x3f, 0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x35, 0x1e,
    0x1f, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x2d, 0x2e, 0x2a, 0x49, 0x4a, 0x4b,
    0x2b, 0x14, 0x1a, 0x08, 0x15, 0x17, 0x1c, 0x18, 0x0c, 0x12, 0x13, 0x2f, 0x30, 0x31, 0x4c,
    0x4d, 0x4e, 0x39, 0x04, 0x16, 0x07, 0x09, 0x0a, 0x0b, 0x0d, 0x0e, 0x0f, 0x33, 0x34, 0x28,
    0xe1, 0x1d, 0x1b, 0x06, 0x19, 0x05, 0x11, 0x10, 0x36, 0x37, 0x38, 0xe5, 0x52, 0xe0, 0xe3,
    0xe2, 0x2c, 0xe6, 0xe7, 0xf0, 0xe4, 0x50, 0x51, 0x4f, 0x32, 0x64, 0x00,
];

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let api = hidapi::HidApi::new().expect("hid api");
    let info = api
        .device_list()
        .find(|d| d.vendor_id() == 0x1038 && d.product_id() == 0x1642 && d.interface_number() == 1)
        .expect("keyboard interface 1");
    let device = info.open_device(&api).expect("open");

    if args.first().map(String::as_str) == Some("clear") {
        let mut report = vec![0u8; 65];
        report[1] = 0x41;
        println!("clear: {:?}", device.write(&report));
        return;
    }
    let colour: Vec<u8> = args.iter().take(3).map(|a| a.parse().expect("0..255")).collect();
    let seconds: u64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(5);
    let keys: Vec<u8> = KEYS.iter().copied().filter(|k| *k != 0).collect();
    let mut report = vec![0u8; 645];
    report[1] = 0x40;
    report[2] = keys.len() as u8;
    for (i, key) in keys.iter().enumerate() {
        report[3 + i * 4..7 + i * 4].copy_from_slice(&[*key, colour[0], colour[1], colour[2]]);
    }
    println!("{} keys, send: {:?}", keys.len(), device.send_feature_report(&report));
    std::thread::sleep(Duration::from_secs(seconds));
    println!("waited {seconds} s");
}
