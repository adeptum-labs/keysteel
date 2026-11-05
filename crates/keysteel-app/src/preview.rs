use keysteel_core::frame::{Frame, HEIGHT, RESERVED_TOP, WIDTH};

/// Colour of a lit OLED pixel, as RGBA.
pub const ON: [u8; 4] = [0xe8, 0xe8, 0xe8, 0xff];
/// Colour of a dark OLED pixel, as RGBA.
pub const OFF: [u8; 4] = [0x10, 0x14, 0x1a, 0xff];
/// Colour of the rows where the keyboard shows its own status text.
pub const BAND: [u8; 4] = [0x2e, 0x34, 0x3e, 0xff];

/// Width and height in pixels of a frame drawn at `scale` times its size.
pub fn preview_size(scale: usize) -> (usize, usize) {
    (WIDTH * scale, HEIGHT * scale)
}

/// Draws `frame` as RGBA bytes, each OLED pixel becoming a `scale` x `scale` block.
pub fn scale_frame_to_rgba(frame: &Frame, scale: usize) -> Vec<u8> {
    let (width, height) = preview_size(scale);
    let mut rgba = Vec::with_capacity(width * height * 4);
    for y in 0..height {
        for x in 0..width {
            let (px, py) = (x / scale, y / scale);
            let colour = if py < RESERVED_TOP {
                &BAND
            } else if frame.get_pixel(px, py) {
                &ON
            } else {
                &OFF
            };
            rgba.extend_from_slice(colour);
        }
    }
    rgba
}

#[cfg(test)]
mod tests {
    use super::*;
    use keysteel_core::frame::Frame;

    fn pixel(rgba: &[u8], width: usize, x: usize, y: usize) -> [u8; 4] {
        let i = (y * width + x) * 4;
        [rgba[i], rgba[i + 1], rgba[i + 2], rgba[i + 3]]
    }

    #[test]
    fn size_is_frame_size_times_scale() {
        assert_eq!(preview_size(3), (384, 120));
    }

    #[test]
    fn buffer_has_four_bytes_per_scaled_pixel() {
        assert_eq!(scale_frame_to_rgba(&Frame::new(), 2).len(), 256 * 80 * 4);
    }

    #[test]
    fn set_pixel_fills_its_whole_scaled_block() {
        let mut frame = Frame::new();
        frame.set_pixel(0, 12, true);
        let rgba = scale_frame_to_rgba(&frame, 2);
        for (x, y) in [(0, 24), (1, 24), (0, 25), (1, 25)] {
            assert_eq!(pixel(&rgba, 256, x, y), ON, "({x},{y})");
        }
        assert_eq!(pixel(&rgba, 256, 2, 24), OFF);
        assert_eq!(pixel(&rgba, 256, 0, 26), OFF);
    }

    #[test]
    fn reserved_rows_are_painted_as_the_status_band() {
        let mut frame = Frame::new();
        frame.set_pixel(5, 0, true);
        let rgba = scale_frame_to_rgba(&frame, 2);
        let band_end = RESERVED_TOP * 2;
        assert_eq!(pixel(&rgba, 256, 10, 0), BAND);
        assert_eq!(pixel(&rgba, 256, 0, band_end - 1), BAND);
        assert_eq!(pixel(&rgba, 256, 0, band_end), OFF);
    }

    #[test]
    fn last_pixel_lands_in_bottom_right_corner() {
        let mut frame = Frame::new();
        frame.set_pixel(127, 39, true);
        let rgba = scale_frame_to_rgba(&frame, 3);
        assert_eq!(pixel(&rgba, 384, 383, 119), ON);
        assert_eq!(pixel(&rgba, 384, 381, 117), ON);
        assert_eq!(pixel(&rgba, 384, 380, 117), OFF);
        assert_eq!(pixel(&rgba, 384, 381, 116), OFF);
    }
}
