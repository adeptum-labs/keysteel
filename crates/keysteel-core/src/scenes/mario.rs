use crate::frame::{Frame, HEIGHT, WIDTH};
use crate::scene::Scene;
use std::time::Duration;

/// Screen column of the middle of the sprite.
const SPRITE_CX: i32 = 40;
/// Half the width of Mario's body, used to keep him out of solid things.
#[cfg(test)]
const HALF_BODY: i32 = 7;
/// Row of the top of the ground; the sprite stands on it.
const GROUND_TOP: i32 = 34;
const SCROLL_PIXELS_PER_SEC: f64 = 60.0;
/// The level repeats after this many pixels of running (a multiple of 8,
/// so the brick pattern lines up).
const PERIOD: i32 = 400;
/// Pictures 0..=2 are the run cycle, the last one is the jump.
const RUN_CYCLE: [usize; 4] = [0, 1, 2, 1];
const JUMP_SPRITE: usize = 3;
/// Pixels of running per step of the run cycle.
const PIXELS_PER_STEP: i32 = 5;
const CLOUD_PIXELS_PER_SEC: f64 = 12.0;
const CLOUD_SPACING: i32 = 80;

/// Something Mario can stand on, given as world columns and the height of
/// its top above the ground.
struct Solid {
    x0: i32,
    x1: i32,
    height: i32,
    pipe: bool,
}

const SOLIDS: [Solid; 2] = [
    Solid { x0: 96, x1: 112, height: 9, pipe: true },
    Solid { x0: 192, x1: 240, height: 8, pipe: false },
];

/// One stretch of Mario's route, measured in running distance.
enum Leg {
    /// Keep the current height until this distance.
    Walk { until: f32 },
    /// Leap to a new height, peaking at `apex`, landing at this distance.
    Arc { until: f32, to: f32, apex: f32 },
}

const ROUTE: [Leg; 7] = [
    Leg::Walk { until: 72.0 },
    Leg::Arc { until: 136.0, to: 0.0, apex: 15.0 },
    Leg::Walk { until: 160.0 },
    Leg::Arc { until: 194.0, to: 8.0, apex: 15.0 },
    Leg::Walk { until: 230.0 },
    Leg::Arc { until: 264.0, to: 0.0, apex: 15.0 },
    Leg::Walk { until: PERIOD as f32 },
];

/// Height of the parabola through (x0, h0) and (x1, h1) peaking at `apex`.
fn arc_height(x0: f32, h0: f32, x1: f32, h1: f32, apex: f32, x: f32) -> f32 {
    let (r0, r1) = ((apex - h0).sqrt(), (apex - h1).sqrt());
    if r0 + r1 == 0.0 {
        return h0;
    }
    let top = (x0 * r1 + x1 * r0) / (r0 + r1);
    let k = if top > x0 { (apex - h0) / ((top - x0) * (top - x0)) } else { (apex - h1) / ((x1 - top) * (x1 - top)) };
    apex - k * (x - top) * (x - top)
}

/// Mario's height above the ground and whether he is in the air, after
/// running `distance` pixels into the level.
fn pose(distance: f32) -> (f32, bool) {
    let (mut x, mut h) = (0.0, 0.0);
    for leg in &ROUTE {
        match *leg {
            Leg::Walk { until } => {
                if distance < until {
                    return (h, false);
                }
                x = until;
            }
            Leg::Arc { until, to, apex } => {
                if distance < until {
                    return (arc_height(x, h, until, to, apex, distance), true);
                }
                x = until;
                h = to;
            }
        }
    }
    (h, false)
}

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

    fn draw_ground(frame: &mut Frame, distance: i32) {
        let top = GROUND_TOP;
        for x in 0..WIDTH as i32 {
            frame.set_pixel(x, top, true);
            frame.set_pixel(x, top + 4, true);
            frame.set_pixel(x, HEIGHT as i32 - 1, true);
            if (x + distance).rem_euclid(8) == 0 {
                for y in top + 1..top + 4 {
                    frame.set_pixel(x, y, true);
                }
            }
            if (x + distance + 4).rem_euclid(8) == 0 {
                for y in top + 5..HEIGHT as i32 - 1 {
                    frame.set_pixel(x, y, true);
                }
            }
        }
    }

    /// Draws a brick platform or a pipe, `shift` pixels from its world spot.
    fn draw_solid(frame: &mut Frame, solid: &Solid, shift: i32) {
        let (x0, x1) = (solid.x0 + shift, solid.x1 + shift);
        let top = GROUND_TOP - solid.height;
        if solid.pipe {
            // Rim, then the shaft down to the ground.
            for x in x0..x1 {
                frame.set_pixel(x, top, true);
                frame.set_pixel(x, top + 3, true);
            }
            for y in top..=top + 3 {
                frame.set_pixel(x0, y, true);
                frame.set_pixel(x1 - 1, y, true);
            }
            for y in top + 4..GROUND_TOP {
                frame.set_pixel(x0 + 1, y, true);
                frame.set_pixel(x1 - 2, y, true);
                frame.set_pixel(x0 + 4, y, y % 2 == 0);
            }
            frame.set_pixel(x0 + 3, top + 1, true);
            frame.set_pixel(x0 + 3, top + 2, true);
        } else {
            for x in x0..x1 {
                frame.set_pixel(x, top, true);
                frame.set_pixel(x, top + 4, true);
                if x == x0 || x == x1 - 1 || (x - x0) % 8 == 0 {
                    for y in top + 1..top + 4 {
                        frame.set_pixel(x, y, true);
                    }
                }
            }
        }
    }

    /// A small outlined cloud with its left edge at `x`.
    fn draw_cloud(frame: &mut Frame, x: i32, y: i32) {
        const CLOUD: [&str; 6] = [
            ".....###......",
            "...##...##.##.",
            "..#........#.#",
            ".#...........#",
            "#............#",
            ".############.",
        ];
        for (dy, row) in CLOUD.iter().enumerate() {
            for (dx, c) in row.chars().enumerate() {
                if c == '#' {
                    frame.set_pixel(x + dx as i32, y + dy as i32, true);
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
        "Super Mario"
    }

    fn animated(&self) -> bool {
        true
    }

    fn render(&mut self, elapsed: Duration, frame: &mut Frame) {
        frame.clear();
        // Only the position inside one repeat matters; f64 keeps it exact
        // after days of uptime.
        let distance = (elapsed.as_secs_f64() * SCROLL_PIXELS_PER_SEC) as u64 % PERIOD as u64;
        let distance = distance as i32;

        // Clouds drift slower than the ground, which gives some depth.
        let cloud = (elapsed.as_secs_f64() * CLOUD_PIXELS_PER_SEC) as u64 % CLOUD_SPACING as u64;
        for i in 0..=WIDTH as i32 / CLOUD_SPACING + 1 {
            Mario::draw_cloud(frame, i * CLOUD_SPACING + 20 - cloud as i32, 3 + (i % 2) * 2);
        }

        Mario::draw_ground(frame, distance);
        for solid in &SOLIDS {
            for lap in [-1, 0, 1] {
                let shift = lap * PERIOD - distance + SPRITE_CX;
                if solid.x1 + shift > 0 && solid.x0 + shift < WIDTH as i32 {
                    Mario::draw_solid(frame, solid, shift);
                }
            }
        }

        let (height, airborne) = pose(distance as f32);
        let index = if airborne {
            JUMP_SPRITE
        } else {
            RUN_CYCLE[(distance / PIXELS_PER_STEP) as usize % RUN_CYCLE.len()]
        };
        let sprite = &self.sprites[index];
        let left = SPRITE_CX - sprite.width as i32 / 2;
        let top = GROUND_TOP - height.round() as i32 - sprite.height as i32;
        // Clear the sprite's box first so clouds do not show through him.
        for y in 0..sprite.height as i32 {
            for x in 0..sprite.width as i32 {
                frame.set_pixel(left + x, top + y, false);
            }
        }
        for y in 0..sprite.height {
            for x in 0..sprite.width {
                if sprite.get(x, y) {
                    frame.set_pixel(left + x as i32, top + y as i32, true);
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
    fn mario_ground_still_scrolls_after_30_days_of_uptime() {
        let thirty_days = 30 * 24 * 3600 * 1000;
        assert_ne!(render_at(thirty_days), render_at(thirty_days + 33));
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

    #[test]
    fn route_starts_and_ends_on_the_ground() {
        assert_eq!(pose(0.0), (0.0, false));
        assert_eq!(pose(PERIOD as f32 - 1.0), (0.0, false));
    }

    #[test]
    fn mario_never_goes_through_a_solid_or_off_screen() {
        let sprite_height = Mario::new().sprites[0].height as f32;
        for distance in 0..PERIOD {
            let (height, _) = pose(distance as f32);
            assert!(height >= 0.0 && height + sprite_height <= GROUND_TOP as f32, "off screen at {distance}");
            for solid in &SOLIDS {
                let overlaps = distance + HALF_BODY > solid.x0 && distance - HALF_BODY < solid.x1;
                if overlaps {
                    assert!(height >= solid.height as f32 - 0.5, "inside a solid at {distance}");
                }
            }
        }
    }

    #[test]
    fn mario_stands_on_the_platform() {
        // Halfway along the brick platform he is up on it, not on the ground.
        let (height, airborne) = pose(210.0);
        assert_eq!((height, airborne), (8.0, false));
    }

    #[test]
    fn the_level_repeats_exactly() {
        // Three laps of the level are exactly 20 s, and the clouds repeat then too.
        assert_eq!(render_at(1000), render_at(21_000));
    }

    #[test]
    fn mario_jumps_with_the_jump_picture() {
        let sprites = parse_sprite_frames(include_str!("../../assets/mario.txt"));
        assert_eq!(sprites.len(), JUMP_SPRITE + 1);
        assert!(pose(100.0).1);
    }
}
