use crate::KeyColour;
use keysteel_core::frame::{Frame, HEIGHT, WIDTH};

/// The feature report is 645 bytes: report id, the live-write header, the
/// 640 picture bytes and zero padding.
pub const REPORT_LEN: usize = 645;

/// Report id, device command 0x1f and the live (direct) write sub-command.
const HEADER: [u8; 3] = [0x00, 0x1f, 0x81];

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

/// Report id, device command 0x40 (set key colours), then how many keys
/// follow and for each its HID usage code and red, green and blue.
const LIGHTS_HEADER: [u8; 2] = [0x00, 0x40];

/// The most keys one lighting report holds.
pub const MAX_LIT_KEYS: usize = (REPORT_LEN - LIGHTS_HEADER.len() - 1) / 4;

/// Feature report that sets the colour of each key in `keys`; any keys
/// beyond what fits are left out.
pub fn encode_lights_report(keys: &[KeyColour]) -> Vec<u8> {
    let keys = &keys[..keys.len().min(MAX_LIT_KEYS)];
    let mut report = vec![0u8; REPORT_LEN];
    report[..LIGHTS_HEADER.len()].copy_from_slice(&LIGHTS_HEADER);
    report[LIGHTS_HEADER.len()] = keys.len() as u8;
    for (i, lit) in keys.iter().enumerate() {
        let at = LIGHTS_HEADER.len() + 1 + i * 4;
        report[at..at + 4].copy_from_slice(&[lit.key, lit.colour.r, lit.colour.g, lit.colour.b]);
    }
    report
}

/// Output report that hands the key lighting back to the keyboard: report
/// id, device command 0x41, zero padding to 65 bytes.
pub fn encode_clear_lights_report() -> Vec<u8> {
    let mut report = vec![0u8; 65];
    report[1] = 0x41;
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use keysteel_core::frame::Frame;

    #[test]
    fn report_has_live_write_header_and_640_page_bytes() {
        let report = encode_report(&Frame::new());
        assert_eq!(report.len(), 645);
        assert_eq!(report.len(), REPORT_LEN);
        assert_eq!(&report[..3], &[0x00, 0x1f, 0x81]);
        assert!(report[3..].iter().all(|b| *b == 0));
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
        assert_eq!(encode_report(&frame)[3], 0x01);
    }

    #[test]
    fn pixel_row_7_is_msb_of_first_page_byte() {
        let mut frame = Frame::new();
        frame.set_pixel(0, 7, true);
        assert_eq!(encode_report(&frame)[3], 0x80);
    }

    #[test]
    fn pixel_row_8_starts_second_page() {
        let mut frame = Frame::new();
        frame.set_pixel(0, 8, true);
        assert_eq!(encode_report(&frame)[3 + 128], 0x01);
    }

    #[test]
    fn pixel_x_127_row_39_is_last_byte_msb_bit_7() {
        let mut frame = Frame::new();
        frame.set_pixel(127, 39, true);
        // page 4, byte 127, row 39 % 8 = 7
        assert_eq!(encode_report(&frame)[3 + 4 * 128 + 127], 0x80);
    }

    #[test]
    fn lights_report_lists_keys_after_the_count() {
        use keysteel_core::lighting::Rgb;
        let keys = [
            KeyColour { key: 0x04, colour: Rgb::new(1, 2, 3) },
            KeyColour { key: 0x32, colour: Rgb::new(4, 5, 6) },
        ];
        let report = encode_lights_report(&keys);
        assert_eq!(report.len(), REPORT_LEN);
        assert_eq!(&report[..11], &[0x00, 0x40, 2, 0x04, 1, 2, 3, 0x32, 4, 5, 6]);
        assert!(report[11..].iter().all(|b| *b == 0));
    }

    #[test]
    fn lights_report_for_no_keys_is_just_the_header() {
        let report = encode_lights_report(&[]);
        assert_eq!(&report[..3], &[0x00, 0x40, 0]);
        assert!(report[3..].iter().all(|b| *b == 0));
    }

    #[test]
    fn lights_report_drops_keys_that_do_not_fit() {
        use keysteel_core::lighting::Rgb;
        let keys = vec![KeyColour { key: 1, colour: Rgb::WHITE }; MAX_LIT_KEYS + 10];
        let report = encode_lights_report(&keys);
        assert_eq!(report.len(), REPORT_LEN);
        assert_eq!(report[2] as usize, MAX_LIT_KEYS);
    }

    #[test]
    fn clear_lights_report_is_id_command_and_padding() {
        let report = encode_clear_lights_report();
        assert_eq!(report.len(), 65);
        assert_eq!(&report[..3], &[0x00, 0x41, 0x00]);
        assert!(report[2..].iter().all(|b| *b == 0));
    }
}
