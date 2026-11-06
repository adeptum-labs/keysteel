//! Small black and white pictures kept as text.

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_reads_hash_as_on() {
        let frames = parse_sprite_frames("#.\n.#\n\n##\n..\n");
        assert_eq!(frames.len(), 2);
        assert!(frames[0].get(0, 0) && !frames[0].get(1, 0) && frames[0].get(1, 1));
        assert!(frames[1].get(1, 0) && !frames[1].get(0, 1));
    }
}
