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
