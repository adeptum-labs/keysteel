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

//! Draws a border and a diagonal on the OLED to check the frame format.

use keysteel_core::frame::{Frame, HEIGHT, WIDTH};
use keysteel_hid::{Device, HidrawDevice};

fn main() {
    let mut device = match HidrawDevice::open() {
        Ok(device) => device,
        Err(e) => {
            eprintln!("cannot open keyboard: {e}");
            std::process::exit(1);
        }
    };
    let mut frame = Frame::new();
    for x in 0..WIDTH as i32 {
        frame.set_pixel(x, 0, true);
        frame.set_pixel(x, HEIGHT as i32 - 1, true);
    }
    for y in 0..HEIGHT as i32 {
        frame.set_pixel(0, y, true);
        frame.set_pixel(WIDTH as i32 - 1, y, true);
        frame.set_pixel(y * WIDTH as i32 / HEIGHT as i32, y, true);
    }
    if let Err(e) = device.send_frame(&frame) {
        eprintln!("send failed: {e}");
        std::process::exit(1);
    }
    println!("frame sent");
}
