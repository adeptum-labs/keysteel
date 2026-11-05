//! Prints a scene as ASCII art: `dump <scene id> <milliseconds>`.

use keysteel_core::frame::{Frame, HEIGHT, WIDTH};
use keysteel_core::scene::scene_by_id;
use std::time::Duration;

fn main() {
    let mut args = std::env::args().skip(1);
    let id = args.next().unwrap_or_else(|| "boing".into());
    let millis: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let Some(mut scene) = scene_by_id(&id) else {
        eprintln!("unknown scene: {id}");
        std::process::exit(1);
    };
    let mut frame = Frame::new();
    scene.render(Duration::from_millis(millis), &mut frame);
    for y in 0..HEIGHT {
        let line: String = (0..WIDTH).map(|x| if frame.get_pixel(x, y) { '#' } else { '.' }).collect();
        println!("{line}");
    }
}
