use crate::frame::{Frame, HEIGHT, WIDTH};
use crate::scene::Scene;
use std::time::Duration;

const SPRITE_X: i32 = 24;
/// Row of the top of the ground; the sprite stands on it.
const GROUND_TOP: i32 = 32;
const FRAME_MILLIS: u128 = 90;
const SCROLL_PIXELS_PER_SEC: f32 = 60.0;

/// One picture of the sprite sheet.
pub struct SpriteFrame {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<bool>,
}

impl SpriteFrame {
    pub fn get(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height && self.pixels[y * self.width + x]
    }
}

/// Reads sprite pictures from text: `#` is a lit pixel, anything else is
/// off, and a blank line separates one picture from the next.
pub fn parse_sprite_frames(text: &str) -> Vec<SpriteFrame> {
    let mut frames = Vec::new();
    let mut rows: Vec<&str> = Vec::new();
    for line in text.lines().chain(std::iter::once("")) {
        if line.trim().is_empty() {
            if !rows.is_empty() {
                frames.push(frame_from_rows(&rows));
                rows.clear();
            }
        } else {
            rows.push(line.trim());
        }
    }
    frames
}

fn frame_from_rows(rows: &[&str]) -> SpriteFrame {
    let width = rows.iter().map(|r| r.chars().count()).max().unwrap_or(0);
    let mut pixels = Vec::with_capacity(width * rows.len());
    for row in rows {
        let mut chars = row.chars();
        for _ in 0..width {
            pixels.push(chars.next() == Some('#'));
        }
    }
    SpriteFrame { width, height: rows.len(), pixels }
}

/// Mario running to the right over scrolling brick ground.
pub struct Mario {
    sprites: Vec<SpriteFrame>,
}

impl Mario {
    pub fn new() -> Mario {
        Mario { sprites: parse_sprite_frames(include_str!("../../assets/mario.txt")) }
    }

    fn draw_ground(frame: &mut Frame, scroll: i32) {
        let top = GROUND_TOP;
        for x in 0..WIDTH as i32 {
            frame.set_pixel(x, top, true);
            frame.set_pixel(x, top + 4, true);
            frame.set_pixel(x, HEIGHT as i32 - 1, true);
            if (x + scroll).rem_euclid(8) == 0 {
                for y in top + 1..top + 4 {
                    frame.set_pixel(x, y, true);
                }
            }
            if (x + scroll + 4).rem_euclid(8) == 0 {
                for y in top + 5..HEIGHT as i32 - 1 {
                    frame.set_pixel(x, y, true);
                }
            }
        }
    }
}

impl Default for Mario {
    fn default() -> Mario {
        Mario::new()
    }
}

impl Scene for Mario {
    fn id(&self) -> &'static str {
        "mario"
    }

    fn name(&self) -> &'static str {
        "Mario running"
    }

    fn animated(&self) -> bool {
        true
    }

    fn render(&mut self, elapsed: Duration, frame: &mut Frame) {
        frame.clear();
        let scroll = (elapsed.as_secs_f32() * SCROLL_PIXELS_PER_SEC) as i32;
        Mario::draw_ground(frame, scroll);

        let index = (elapsed.as_millis() / FRAME_MILLIS) as usize % self.sprites.len();
        let sprite = &self.sprites[index];
        let top = GROUND_TOP - sprite.height as i32;
        for y in 0..sprite.height {
            for x in 0..sprite.width {
                if sprite.get(x, y) {
                    frame.set_pixel(SPRITE_X + x as i32, top + y as i32, true);
                }
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
        Mario::new().render(Duration::from_millis(ms), &mut frame);
        frame
    }

    #[test]
    fn mario_sprite_asset_parses() {
        let frames = parse_sprite_frames(include_str!("../../assets/mario.txt"));
        assert!(frames.len() >= 2);
        let (w, h) = (frames[0].width, frames[0].height);
        assert!(frames.iter().all(|f| f.width == w && f.height == h));
        assert!(h <= 20 && w <= 20);
    }

    #[test]
    fn parse_reads_hash_as_on() {
        let frames = parse_sprite_frames("#.\n.#\n\n##\n..\n");
        assert_eq!(frames.len(), 2);
        assert!(frames[0].get(0, 0) && !frames[0].get(1, 0) && frames[0].get(1, 1));
        assert!(frames[1].get(1, 0) && !frames[1].get(0, 1));
    }

    #[test]
    fn mario_sprite_frames_differ() {
        let frames = parse_sprite_frames(include_str!("../../assets/mario.txt"));
        assert_ne!(frames[0].pixels, frames[1].pixels);
    }

    #[test]
    fn mario_is_deterministic() {
        assert_eq!(render_at(450), render_at(450));
    }

    #[test]
    fn mario_runs() {
        assert_ne!(render_at(0), render_at(100));
    }

    #[test]
    fn mario_stays_in_view_at_any_time() {
        for ms in (0..10_000).step_by(97) {
            assert!(render_at(ms).as_bytes().iter().any(|b| *b != 0), "blank at {ms} ms");
        }
    }
}
