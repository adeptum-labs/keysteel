mod hidraw;
mod mock;
mod report;

pub use hidraw::{is_keyboard_hid_id, HidrawDevice};
pub use mock::MockDevice;
pub use report::{encode_report, encode_reset_report, REPORT_LEN};

use keysteel_core::frame::Frame;

#[derive(Debug)]
pub enum DeviceError {
    NotFound,
    Hid(String),
}

impl std::fmt::Display for DeviceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceError::NotFound => write!(f, "keyboard not found"),
            DeviceError::Hid(message) => write!(f, "hid error: {message}"),
        }
    }
}

impl std::error::Error for DeviceError {}

/// A display that can show one frame at a time.
pub trait Device {
    fn send_frame(&mut self, frame: &Frame) -> Result<(), DeviceError>;
    /// Gives the screen back to the keyboard's own display.
    fn reset(&mut self) -> Result<(), DeviceError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_records_frames() {
        let mut device = MockDevice::default();
        let mut frame = Frame::new();
        frame.set_pixel(1, 2, true);
        device.send_frame(&frame).unwrap();
        device.send_frame(&Frame::new()).unwrap();
        assert_eq!(device.frames.len(), 2);
        assert_eq!(device.frames[0], frame);
    }

    #[test]
    fn mock_counts_resets() {
        let mut device = MockDevice::default();
        device.reset().unwrap();
        device.reset().unwrap();
        assert_eq!(device.resets, 2);
        assert!(device.frames.is_empty());
    }
}
