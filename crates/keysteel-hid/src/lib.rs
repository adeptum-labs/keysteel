mod mock;
mod report;

pub use mock::MockDevice;
pub use report::{encode_report, REPORT_LEN};

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
}
