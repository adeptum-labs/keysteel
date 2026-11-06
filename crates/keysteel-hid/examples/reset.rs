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

//! Hands the OLED back to the keyboard: sends the display reset report.
//! With the argument `blank` it first sends an empty picture.

use keysteel_core::frame::Frame;
use keysteel_hid::{Device, HidrawDevice};

/// Report id, device command 0x38, sub-command 0x83 (store the picture the
/// keyboard shows by itself) and the "reset to default" flag, then a blank
/// picture.
fn restore_default_report() -> Vec<u8> {
    let mut report = vec![0u8; 645];
    report[1] = 0x38;
    report[2] = 0x83;
    report[3] = 0x01;
    report
}

fn main() {
    let mut device = match HidrawDevice::open() {
        Ok(device) => device,
        Err(e) => {
            eprintln!("cannot open keyboard: {e}");
            std::process::exit(1);
        }
    };
    if std::env::args().nth(1).as_deref() == Some("default") {
        let api = hidapi::HidApi::new().expect("hid api");
        let info = api
            .device_list()
            .find(|d| d.vendor_id() == 0x1038 && d.product_id() == 0x1642 && d.interface_number() == 1)
            .expect("keyboard interface 1");
        let handle = info.open_device(&api).expect("open");
        println!("restore default: {:?}", handle.send_feature_report(&restore_default_report()));
        return;
    }
    if std::env::args().nth(1).as_deref() == Some("blank") {
        println!("blank: {:?}", device.send_frame(&Frame::new()));
        std::thread::sleep(std::time::Duration::from_secs(3));
    }
    println!("reset: {:?}", device.reset());
}
