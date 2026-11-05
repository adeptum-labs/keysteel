pub const WIDTH: usize = 128;
pub const HEIGHT: usize = 40;
pub const BYTES: usize = WIDTH * HEIGHT / 8;
/// The keyboard draws its own status text over the top rows; scenes keep
/// to the rows below.
pub const RESERVED_TOP: usize = 12;

/// A 128x40 one-bit-per-pixel image, row-major, most significant bit first.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Frame {
    bytes: [u8; BYTES],
}

impl Frame {
    pub fn new() -> Frame {
        Frame { bytes: [0; BYTES] }
    }

    /// Sets a pixel; coordinates outside the frame are ignored.
    pub fn set_pixel(&mut self, x: i32, y: i32, on: bool) {
        if x < 0 || y < 0 || x as usize >= WIDTH || y as usize >= HEIGHT {
            return;
        }
        let index = y as usize * WIDTH + x as usize;
        let mask = 0x80 >> (index % 8);
        if on {
            self.bytes[index / 8] |= mask;
        } else {
            self.bytes[index / 8] &= !mask;
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> bool {
        if x >= WIDTH || y >= HEIGHT {
            return false;
        }
        let index = y * WIDTH + x;
        self.bytes[index / 8] & (0x80 >> (index % 8)) != 0
    }

    pub fn clear(&mut self) {
        self.bytes = [0; BYTES];
    }

    pub fn as_bytes(&self) -> &[u8; BYTES] {
        &self.bytes
    }
}

impl Default for Frame {
    fn default() -> Frame {
        Frame::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixel_0_0_is_msb_of_byte_0() {
        let mut f = Frame::new();
        f.set_pixel(0, 0, true);
        assert_eq!(f.as_bytes()[0], 0x80);
    }

    #[test]
    fn pixel_127_0_is_lsb_of_byte_15() {
        let mut f = Frame::new();
        f.set_pixel(127, 0, true);
        assert_eq!(f.as_bytes()[15], 0x01);
    }

    #[test]
    fn pixel_0_1_starts_byte_16() {
        let mut f = Frame::new();
        f.set_pixel(0, 1, true);
        assert_eq!(f.as_bytes()[16], 0x80);
    }

    #[test]
    fn out_of_range_is_ignored() {
        let mut f = Frame::new();
        f.set_pixel(-1, 0, true);
        f.set_pixel(128, 0, true);
        f.set_pixel(0, 40, true);
        assert!(f.as_bytes().iter().all(|b| *b == 0));
    }

    #[test]
    fn clear_resets() {
        let mut f = Frame::new();
        f.set_pixel(5, 5, true);
        f.clear();
        assert!(f.as_bytes().iter().all(|b| *b == 0));
    }
}
