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

use crate::frame::{Frame, HEIGHT, WIDTH};
use crate::scene::Scene;
use std::time::Duration;

/// How often the processor load is read. Frames in between reuse the result.
const SAMPLE_INTERVAL: Duration = Duration::from_millis(500);

/// The total is drawn in the left part, the core matrix in the right part.
const LEFT_WIDTH: i32 = 56;
const MATRIX_LEFT: usize = 60;
const MATRIX_WIDTH: usize = WIDTH - MATRIX_LEFT;
const MAX_CELL: usize = 12;

const DIGITS_TOP: i32 = 6;
const BAR_TOP: i32 = 27;
const BAR_HEIGHT: i32 = 5;

const SCALE: i32 = 3;
const GLYPH_WIDTH: i32 = 3 * SCALE;
const GLYPH_ADVANCE: i32 = GLYPH_WIDTH + SCALE;

/// 3x5 glyphs for 0-9 and %, one byte per row, three bits wide.
const GLYPHS: [[u8; 5]; 11] = [
    [0b111, 0b101, 0b101, 0b101, 0b111],
    [0b010, 0b110, 0b010, 0b010, 0b111],
    [0b111, 0b001, 0b111, 0b100, 0b111],
    [0b111, 0b001, 0b111, 0b001, 0b111],
    [0b101, 0b101, 0b111, 0b001, 0b001],
    [0b111, 0b100, 0b111, 0b001, 0b111],
    [0b111, 0b100, 0b111, 0b101, 0b111],
    [0b111, 0b001, 0b001, 0b001, 0b001],
    [0b111, 0b101, 0b111, 0b101, 0b111],
    [0b111, 0b101, 0b111, 0b001, 0b111],
    [0b101, 0b001, 0b010, 0b100, 0b101],
];

/// Busy and total jiffies of one processor line in /proc/stat.
type Counters = (u64, u64);

/// One reading of /proc/stat: the aggregate line and every active core.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Stat {
    total: Counters,
    cores: Vec<Counters>,
}

fn parse_counters(fields: &[&str]) -> Option<Counters> {
    let values: Vec<u64> = fields.iter().map(|f| f.parse().ok()).collect::<Option<_>>()?;
    if values.len() < 4 {
        return None;
    }
    // user nice system idle iowait irq softirq steal; the guest columns are
    // already part of user and nice.
    let total: u64 = values.iter().take(8).sum();
    let idle = values[3] + values.get(4).copied().unwrap_or(0);
    Some((total.saturating_sub(idle), total))
}

/// Reads the `cpu` line and the `cpuN` lines. Offline cores have no line,
/// so only the active ones are returned. None when there is no `cpu` line.
fn parse_stat(text: &str) -> Option<Stat> {
    let mut total = None;
    let mut cores = Vec::new();
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let Some(name) = parts.next() else {
            continue;
        };
        let Some(rest) = name.strip_prefix("cpu") else {
            continue;
        };
        let fields: Vec<&str> = parts.collect();
        let Some(counters) = parse_counters(&fields) else {
            continue;
        };
        if rest.is_empty() {
            total = Some(counters);
        } else if rest.bytes().all(|b| b.is_ascii_digit()) {
            cores.push(counters);
        }
    }
    Some(Stat { total: total?, cores })
}

/// The share of time that was busy between two readings, 0.0 to 1.0.
fn usage(before: Counters, after: Counters) -> f64 {
    let total = after.1.saturating_sub(before.1);
    if total == 0 {
        return 0.0;
    }
    let busy = after.0.saturating_sub(before.0);
    (busy as f64 / total as f64).clamp(0.0, 1.0)
}

fn smooth(old: f64, measured: f64) -> f64 {
    0.5 * old + 0.5 * measured
}

fn percent(usage: f64) -> u32 {
    (usage.clamp(0.0, 1.0) * 100.0).round() as u32
}

/// How many pixel rows of `rows` a load fills, rounded to the nearest.
fn fill_rows(usage: f64, rows: usize) -> usize {
    (usage.clamp(0.0, 1.0) * rows as f64).round() as usize
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Cell {
    x: i32,
    y: i32,
    size: usize,
}

/// Places `count` cells in the matrix area: the columns that give the
/// largest square cells, centred, one pixel apart.
fn grid(count: usize) -> Vec<Cell> {
    if count == 0 {
        return Vec::new();
    }
    let mut best_cols = 1;
    let mut best_pitch = 0;
    for cols in 1..=count {
        let rows = count.div_ceil(cols);
        let pitch = (MATRIX_WIDTH / cols).min(HEIGHT / rows).min(MAX_CELL);
        if pitch > best_pitch {
            best_pitch = pitch;
            best_cols = cols;
        }
    }
    let cols = best_cols;
    let rows = count.div_ceil(cols);
    let pitch = best_pitch.max(1);
    let size = if pitch >= 2 { pitch - 1 } else { 1 };
    let grid_width = cols * pitch - (pitch - size);
    let grid_height = rows * pitch - (pitch - size);
    let left = MATRIX_LEFT + MATRIX_WIDTH.saturating_sub(grid_width) / 2;
    let top = HEIGHT.saturating_sub(grid_height) / 2;
    (0..count)
        .map(|i| Cell {
            x: (left + (i % cols) * pitch) as i32,
            y: (top + (i / cols) * pitch) as i32,
            size,
        })
        .collect()
}

fn fill_rect(frame: &mut Frame, x: i32, y: i32, width: i32, height: i32) {
    for yy in y..y + height {
        for xx in x..x + width {
            frame.set_pixel(xx, yy, true);
        }
    }
}

/// Draws one core: an outline with the load filling it from the bottom.
/// Cells too small for an outline are filled without one.
fn draw_cell(frame: &mut Frame, cell: Cell, load: f64) {
    let size = cell.size as i32;
    if cell.size >= 4 {
        let inner = cell.size - 2;
        for i in 0..size {
            frame.set_pixel(cell.x + i, cell.y, true);
            frame.set_pixel(cell.x + i, cell.y + size - 1, true);
            frame.set_pixel(cell.x, cell.y + i, true);
            frame.set_pixel(cell.x + size - 1, cell.y + i, true);
        }
        let filled = fill_rows(load, inner) as i32;
        fill_rect(frame, cell.x + 1, cell.y + size - 1 - filled, size - 2, filled);
    } else {
        let filled = fill_rows(load, cell.size) as i32;
        fill_rect(frame, cell.x, cell.y + size - filled, size, filled);
    }
}

fn text_width(chars: usize) -> i32 {
    chars as i32 * GLYPH_ADVANCE - SCALE
}

/// Draws `text` (digits and %) in big pixels, its right edge at `right`.
fn draw_text(frame: &mut Frame, text: &str, right: i32, top: i32) {
    let mut x = right - text_width(text.chars().count());
    for ch in text.chars() {
        let index = match ch {
            '0'..='9' => ch as usize - '0' as usize,
            '%' => 10,
            _ => {
                x += GLYPH_ADVANCE;
                continue;
            }
        };
        for (row, bits) in GLYPHS[index].iter().enumerate() {
            for col in 0..3 {
                if bits >> (2 - col) & 1 == 1 {
                    fill_rect(frame, x + col * SCALE, top + row as i32 * SCALE, SCALE, SCALE);
                }
            }
        }
        x += GLYPH_ADVANCE;
    }
}

/// A horizontal bar with an outline, filled from the left by `load`.
fn draw_bar(frame: &mut Frame, load: f64) {
    let width = LEFT_WIDTH;
    for x in 0..width {
        frame.set_pixel(x, BAR_TOP, true);
        frame.set_pixel(x, BAR_TOP + BAR_HEIGHT - 1, true);
    }
    for y in 0..BAR_HEIGHT {
        frame.set_pixel(0, BAR_TOP + y, true);
        frame.set_pixel(width - 1, BAR_TOP + y, true);
    }
    let filled = fill_rows(load, (width - 2) as usize) as i32;
    fill_rect(frame, 1, BAR_TOP + 1, filled, BAR_HEIGHT - 2);
}

type Source = Box<dyn FnMut() -> Option<String> + Send>;

/// The processor load: the total as a number and a bar, and one cell per
/// active core filled by how busy it is. Unlike the other scenes it reads
/// live data, from /proc/stat.
pub struct Cpu {
    source: Source,
    previous: Option<Stat>,
    sampled_at: Option<Duration>,
    total: f64,
    cores: Vec<f64>,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu::with_source(|| std::fs::read_to_string("/proc/stat").ok())
    }

    fn with_source(source: impl FnMut() -> Option<String> + Send + 'static) -> Cpu {
        Cpu {
            source: Box::new(source),
            previous: None,
            sampled_at: None,
            total: 0.0,
            cores: Vec::new(),
        }
    }

    fn update(&mut self, elapsed: Duration) {
        if let Some(at) = self.sampled_at {
            if elapsed >= at && elapsed - at < SAMPLE_INTERVAL {
                return;
            }
        }
        self.sampled_at = Some(elapsed);
        let Some(current) = (self.source)().as_deref().and_then(parse_stat) else {
            self.previous = None;
            self.total = 0.0;
            self.cores.clear();
            return;
        };
        match &self.previous {
            Some(before) => {
                if self.cores.len() != current.cores.len() {
                    self.cores = vec![0.0; current.cores.len()];
                }
                self.total = smooth(self.total, usage(before.total, current.total));
                for (i, load) in self.cores.iter_mut().enumerate() {
                    let measured = before.cores.get(i).map_or(0.0, |b| usage(*b, current.cores[i]));
                    *load = smooth(*load, measured);
                }
            }
            None => {
                self.total = 0.0;
                self.cores = vec![0.0; current.cores.len()];
            }
        }
        self.previous = Some(current);
    }
}

impl Default for Cpu {
    fn default() -> Cpu {
        Cpu::new()
    }
}

impl Scene for Cpu {
    fn id(&self) -> &'static str {
        "cpu"
    }

    fn name(&self) -> &'static str {
        "CPU usage"
    }

    fn animated(&self) -> bool {
        true
    }

    fn render(&mut self, elapsed: Duration, frame: &mut Frame) {
        frame.clear();
        self.update(elapsed);
        draw_text(frame, &format!("{}%", percent(self.total)), LEFT_WIDTH, DIGITS_TOP);
        draw_bar(frame, self.total);
        for (cell, load) in grid(self.cores.len()).into_iter().zip(&self.cores) {
            draw_cell(frame, cell, *load);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// A /proc/stat text from (busy, idle) pairs, one per core.
    fn stat_text(cores: &[(u64, u64)]) -> String {
        let busy: u64 = cores.iter().map(|c| c.0).sum();
        let idle: u64 = cores.iter().map(|c| c.1).sum();
        let mut text = format!("cpu  {busy} 0 0 {idle} 0 0 0 0 0 0\n");
        for (i, (b, id)) in cores.iter().enumerate() {
            text.push_str(&format!("cpu{i} {b} 0 0 {id} 0 0 0 0 0 0\n"));
        }
        text.push_str("intr 12345 1 2 3\nctxt 99\n");
        text
    }

    fn scripted(texts: Vec<String>) -> Cpu {
        let mut next = 0;
        Cpu::with_source(move || {
            let text = texts[next.min(texts.len() - 1)].clone();
            next += 1;
            Some(text)
        })
    }

    fn lit_in(frame: &Frame, x: i32, y: i32, width: i32, height: i32) -> usize {
        let mut count = 0;
        for yy in y..y + height {
            for xx in x..x + width {
                if frame.get_pixel(xx as usize, yy as usize) {
                    count += 1;
                }
            }
        }
        count
    }

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[test]
    fn parse_stat_reads_the_total_and_every_core() {
        let stat = parse_stat(&stat_text(&[(30, 70), (10, 90)])).unwrap();
        assert_eq!(stat.total, (40, 200));
        assert_eq!(stat.cores, vec![(30, 100), (10, 100)]);
    }

    #[test]
    fn parse_stat_counts_iowait_as_idle() {
        let stat = parse_stat("cpu 10 0 10 70 10 0 0 0 0 0\ncpu0 10 0 10 70 10 0 0 0 0 0\n").unwrap();
        assert_eq!(stat.total, (20, 100));
    }

    #[test]
    fn parse_stat_skips_an_offline_core() {
        let text = "cpu 3 0 0 7 0 0 0 0 0 0\ncpu0 1 0 0 4 0 0 0 0 0 0\ncpu2 2 0 0 3 0 0 0 0 0 0\n";
        let stat = parse_stat(text).unwrap();
        assert_eq!(stat.cores, vec![(1, 5), (2, 5)]);
    }

    #[test]
    fn parse_stat_without_a_total_line_is_none() {
        assert_eq!(parse_stat(""), None);
        assert_eq!(parse_stat("garbage\nintr 1 2\n"), None);
        assert_eq!(parse_stat("cpu0 1 0 0 4 0 0 0 0 0 0\n"), None);
    }

    #[test]
    fn usage_is_the_busy_share_of_the_new_time() {
        assert_eq!(usage((10, 100), (60, 200)), 0.5);
        assert_eq!(usage((10, 100), (10, 200)), 0.0);
        assert_eq!(usage((10, 100), (110, 200)), 1.0);
    }

    #[test]
    fn usage_without_new_time_is_zero() {
        assert_eq!(usage((10, 100), (10, 100)), 0.0);
        assert_eq!(usage((10, 100), (5, 50)), 0.0);
    }

    #[test]
    fn percent_is_rounded_and_clamped() {
        assert_eq!(percent(0.0), 0);
        assert_eq!(percent(0.426), 43);
        assert_eq!(percent(1.7), 100);
    }

    #[test]
    fn load_is_smoothed_between_samples() {
        let mut cpu = scripted(vec![
            stat_text(&[(0, 0), (0, 0)]),
            stat_text(&[(100, 0), (0, 100)]),
            stat_text(&[(200, 0), (0, 200)]),
        ]);
        let mut frame = Frame::new();
        cpu.render(ms(0), &mut frame);
        assert_eq!(cpu.cores, vec![0.0, 0.0]);
        cpu.render(ms(500), &mut frame);
        assert_eq!(cpu.cores, vec![0.5, 0.0]);
        assert_eq!(cpu.total, 0.25);
        cpu.render(ms(1000), &mut frame);
        assert_eq!(cpu.cores, vec![0.75, 0.0]);
    }

    #[test]
    fn the_source_is_read_at_most_twice_a_second() {
        let reads = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&reads);
        let mut cpu = Cpu::with_source(move || {
            counter.fetch_add(1, Ordering::SeqCst);
            Some(stat_text(&[(0, 0)]))
        });
        let mut frame = Frame::new();
        for t in (0..500).step_by(33) {
            cpu.render(ms(t), &mut frame);
        }
        assert_eq!(reads.load(Ordering::SeqCst), 1);
        cpu.render(ms(500), &mut frame);
        assert_eq!(reads.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn a_missing_or_garbled_source_shows_zero_and_no_cores() {
        for source in [None, Some(String::new()), Some("not a stat file".to_string())] {
            let mut cpu = Cpu::with_source(move || source.clone());
            let mut frame = Frame::new();
            cpu.render(ms(0), &mut frame);
            cpu.render(ms(600), &mut frame);
            assert!(cpu.cores.is_empty());
            let mut zero = Frame::new();
            draw_text(&mut zero, "0%", LEFT_WIDTH, DIGITS_TOP);
            draw_bar(&mut zero, 0.0);
            assert_eq!(frame, zero);
        }
    }

    #[test]
    fn the_core_count_changing_restarts_the_cores() {
        let mut cpu = scripted(vec![
            stat_text(&[(0, 0), (0, 0)]),
            stat_text(&[(50, 50), (50, 50)]),
            stat_text(&[(100, 100), (100, 100), (0, 0)]),
        ]);
        let mut frame = Frame::new();
        cpu.render(ms(0), &mut frame);
        cpu.render(ms(500), &mut frame);
        assert_eq!(cpu.cores.len(), 2);
        cpu.render(ms(1000), &mut frame);
        assert_eq!(cpu.cores.len(), 3);
    }

    #[test]
    fn the_grid_keeps_every_cell_in_the_matrix_area_without_overlap() {
        for count in [1, 2, 4, 6, 12, 16, 24, 64, 128] {
            let cells = grid(count);
            assert_eq!(cells.len(), count);
            for cell in &cells {
                assert!(cell.x >= MATRIX_LEFT as i32, "{count}: {cell:?}");
                assert!(cell.x + cell.size as i32 <= WIDTH as i32, "{count}: {cell:?}");
                assert!(cell.y >= 0, "{count}: {cell:?}");
                assert!(cell.y + cell.size as i32 <= HEIGHT as i32, "{count}: {cell:?}");
            }
            for (i, a) in cells.iter().enumerate() {
                for b in &cells[i + 1..] {
                    let apart = a.x + a.size as i32 <= b.x
                        || b.x + b.size as i32 <= a.x
                        || a.y + a.size as i32 <= b.y
                        || b.y + b.size as i32 <= a.y;
                    assert!(apart, "{count}: {a:?} overlaps {b:?}");
                }
            }
        }
    }

    #[test]
    fn four_cores_get_big_cells_and_sixteen_get_smaller_ones() {
        assert!(grid(4)[0].size > grid(16)[0].size);
        assert!(grid(16)[0].size >= 4);
    }

    #[test]
    fn a_cell_fills_from_the_bottom_with_the_load() {
        let cell = Cell { x: 10, y: 4, size: 8 };
        let inner = |frame: &Frame| lit_in(frame, 11, 5, 6, 6);
        for (load, rows) in [(0.0, 0), (0.5, 3), (1.0, 6)] {
            let mut frame = Frame::new();
            draw_cell(&mut frame, cell, load);
            assert_eq!(inner(&frame), rows * 6, "load {load}");
            // The outline is always there.
            assert!(frame.get_pixel(10, 4) && frame.get_pixel(17, 11));
            if rows > 0 {
                assert!(frame.get_pixel(11, 10), "the bottom row fills first");
            }
            if rows < 6 {
                assert!(!frame.get_pixel(11, 5), "the top row fills last");
            }
        }
    }

    #[test]
    fn a_tiny_cell_is_filled_without_an_outline() {
        let cell = Cell { x: 100, y: 10, size: 2 };
        let mut frame = Frame::new();
        draw_cell(&mut frame, cell, 0.5);
        assert_eq!(lit_in(&frame, 100, 10, 2, 2), 2);
        assert!(frame.get_pixel(100, 11) && !frame.get_pixel(100, 10));
    }

    #[test]
    fn the_total_is_drawn_as_big_digits() {
        let lit = |text: &str| {
            let mut frame = Frame::new();
            draw_text(&mut frame, text, LEFT_WIDTH, DIGITS_TOP);
            frame
        };
        let seven = lit("7%");
        let x7 = LEFT_WIDTH - text_width(2);
        // The top row of a 7 is solid, the lower left is empty, the stem is on the right.
        assert!(seven.get_pixel(x7 as usize, DIGITS_TOP as usize));
        assert!(!seven.get_pixel(x7 as usize, (DIGITS_TOP + 6) as usize));
        assert!(seven.get_pixel((x7 + 7) as usize, (DIGITS_TOP + 6) as usize));
        let all: Vec<Frame> = ["0%", "7%", "42%", "100%"].iter().map(|t| lit(t)).collect();
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                assert_ne!(a, b);
            }
        }
        // Even 100% fits in the left part.
        assert!(text_width(4) <= LEFT_WIDTH);
        assert_eq!(lit_in(&all[3], LEFT_WIDTH, 0, WIDTH as i32 - LEFT_WIDTH, HEIGHT as i32), 0);
    }

    #[test]
    fn the_bar_fills_from_the_left() {
        let mut frame = Frame::new();
        draw_bar(&mut frame, 0.5);
        let inside_y = (BAR_TOP + 2) as usize;
        assert!(frame.get_pixel(1, inside_y));
        assert!(frame.get_pixel(27, inside_y));
        assert!(!frame.get_pixel(40, inside_y));
        assert!(frame.get_pixel(0, inside_y) && frame.get_pixel((LEFT_WIDTH - 1) as usize, inside_y));
    }

    #[test]
    fn the_scene_draws_a_cell_per_core() {
        let mut cpu = scripted(vec![stat_text(&[(0, 0); 4])]);
        let mut frame = Frame::new();
        cpu.render(ms(0), &mut frame);
        for cell in grid(4) {
            assert!(frame.get_pixel(cell.x as usize, cell.y as usize));
        }
    }
}
