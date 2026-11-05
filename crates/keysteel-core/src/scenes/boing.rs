use crate::frame::{Frame, HEIGHT, WIDTH};
use crate::scene::Scene;
use std::f32::consts::FRAC_PI_4;
use std::time::Duration;

const RADIUS: f32 = 10.0;
/// Row of the ground under the ball; the ball rests with its bottom here.
const GROUND_Y: f32 = 36.0;
const SHADOW_Y: i32 = 38;
const MAX_RISE: f32 = 15.0;
const BOUNCE_SECS: f32 = 1.2;
const TRIP_SECS: f32 = 4.0;
const SPIN_RAD_PER_SEC: f32 = 2.0;

/// The Amiga demo ball: a checkered sphere that spins, bounces and drifts
/// from side to side over a shadow.
pub struct Boing;

impl Boing {
    pub fn new() -> Boing {
        Boing
    }
}

impl Default for Boing {
    fn default() -> Boing {
        Boing::new()
    }
}

impl Scene for Boing {
    fn id(&self) -> &'static str {
        "boing"
    }

    fn name(&self) -> &'static str {
        "Boing ball"
    }

    fn animated(&self) -> bool {
        true
    }

    fn render(&mut self, elapsed: Duration, frame: &mut Frame) {
        frame.clear();
        // f64 keeps the motion smooth after days of uptime; each cycle is
        // wrapped before it drops to f32.
        let t = elapsed.as_secs_f64();

        let trip = (t / TRIP_SECS as f64).fract() as f32;
        let across = if trip < 0.5 { trip * 2.0 } else { 2.0 - trip * 2.0 };
        let margin = RADIUS + 1.0;
        let cx = margin + across * (WIDTH as f32 - 2.0 * margin);

        let phase = (t / BOUNCE_SECS as f64).fract() as f32;
        let height = 1.0 - (2.0 * phase - 1.0).powi(2);
        let cy = GROUND_Y - RADIUS - height * MAX_RISE;

        let spin = ((t * SPIN_RAD_PER_SEC as f64) % std::f64::consts::TAU) as f32;

        let shadow_half = RADIUS * (1.0 - 0.5 * height);
        let mut x = (cx - shadow_half).round() as i32;
        while x as f32 <= cx + shadow_half {
            frame.set_pixel(x, SHADOW_Y, true);
            x += 1;
        }

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let dx = x as f32 + 0.5 - cx;
                let dy = y as f32 + 0.5 - cy;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist > RADIUS {
                    continue;
                }
                let on = if dist > RADIUS - 1.0 {
                    true
                } else {
                    let nx = dx / RADIUS;
                    let ny = dy / RADIUS;
                    let nz = (1.0 - nx * nx - ny * ny).max(0.0).sqrt();
                    let longitude = nx.atan2(nz) + spin;
                    let latitude = ny.asin();
                    let column = (longitude / FRAC_PI_4).floor() as i32;
                    let row = (latitude / FRAC_PI_4).floor() as i32;
                    (column + row).rem_euclid(2) == 0
                };
                frame.set_pixel(x as i32, y as i32, on);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::Scene;
    use std::time::Duration;

    fn render_at(ms: u64) -> Frame {
        let mut frame = Frame::new();
        Boing::new().render(Duration::from_millis(ms), &mut frame);
        frame
    }

    #[test]
    fn boing_is_deterministic() {
        assert_eq!(render_at(700), render_at(700));
    }

    #[test]
    fn boing_still_moves_after_30_days_of_uptime() {
        let thirty_days = 30 * 24 * 3600 * 1000;
        assert_ne!(render_at(thirty_days), render_at(thirty_days + 33));
    }

    #[test]
    fn boing_ball_moves() {
        assert_ne!(render_at(0), render_at(500));
    }

    #[test]
    fn boing_draws_something_at_any_time() {
        for ms in (0..8000).step_by(137) {
            assert!(render_at(ms).as_bytes().iter().any(|b| *b != 0), "blank at {ms} ms");
        }
    }

    #[test]
    fn boing_clears_previous_content() {
        let mut frame = Frame::new();
        for x in 0..128 {
            frame.set_pixel(x, 20, true);
        }
        let mut fresh = Frame::new();
        Boing::new().render(Duration::from_millis(300), &mut frame);
        Boing::new().render(Duration::from_millis(300), &mut fresh);
        assert_eq!(frame, fresh);
    }
}
