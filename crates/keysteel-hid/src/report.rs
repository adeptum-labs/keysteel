use keysteel_core::frame::{Frame, HEIGHT, WIDTH};

/// Feature report: report id, four header bytes, then 128 * 5 page bytes.
pub const REPORT_LEN: usize = 5 + WIDTH * HEIGHT / 8;

const HEADER: [u8; 5] = [0x00, 0x38, 0x83, 0x00, 0x00];

/// Packs a frame the way the Apex Pro TKL Gen 3 OLED expects it: pages of
/// eight rows, one byte per column, top row of a page in the lowest bit.
pub fn encode_report(frame: &Frame) -> Vec<u8> {
    let mut report = vec![0u8; REPORT_LEN];
    report[..HEADER.len()].copy_from_slice(&HEADER);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if frame.get_pixel(x, y) {
                report[HEADER.len() + y / 8 * WIDTH + x] |= 1 << (y % 8);
            }
        }
    }
    report
}

/// Output report that hands the screen back to the keyboard's own display:
/// report id, device command 0x1f, sub-command 0x82, zero padding to 65 bytes.
pub fn encode_reset_report() -> Vec<u8> {
    let mut report = vec![0u8; 65];
    report[1] = 0x1f;
    report[2] = 0x82;
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use keysteel_core::frame::Frame;

    #[test]
    fn report_has_id_header_and_640_page_bytes() {
        let report = encode_report(&Frame::new());
        assert_eq!(report.len(), 645);
        assert_eq!(report.len(), REPORT_LEN);
        assert_eq!(&report[..5], &[0x00, 0x38, 0x83, 0x00, 0x00]);
        assert!(report[5..].iter().all(|b| *b == 0));
    }

    #[test]
    fn reset_report_is_id_command_and_padding() {
        let report = encode_reset_report();
        assert_eq!(report.len(), 65);
        assert_eq!(&report[..3], &[0x00, 0x1f, 0x82]);
        assert!(report[3..].iter().all(|b| *b == 0));
    }

    #[test]
    fn top_left_pixel_is_lsb_of_first_page_byte() {
        let mut frame = Frame::new();
        frame.set_pixel(0, 0, true);
        assert_eq!(encode_report(&frame)[5], 0x01);
    }

    #[test]
    fn pixel_row_7_is_msb_of_first_page_byte() {
        let mut frame = Frame::new();
        frame.set_pixel(0, 7, true);
        assert_eq!(encode_report(&frame)[5], 0x80);
    }

    #[test]
    fn pixel_row_8_starts_second_page() {
        let mut frame = Frame::new();
        frame.set_pixel(0, 8, true);
        assert_eq!(encode_report(&frame)[5 + 128], 0x01);
    }

    #[test]
    fn pixel_x_127_row_39_is_last_byte_msb_bit_7() {
        let mut frame = Frame::new();
        frame.set_pixel(127, 39, true);
        // page 4, byte 127, row 39 % 8 = 7
        assert_eq!(encode_report(&frame)[5 + 4 * 128 + 127], 0x80);
    }
}
