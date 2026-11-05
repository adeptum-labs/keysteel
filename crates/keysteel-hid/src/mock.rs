use crate::{Device, DeviceError};
use keysteel_core::frame::Frame;

/// Records every frame it is sent. Used in tests.
#[derive(Default)]
pub struct MockDevice {
    pub frames: Vec<Frame>,
    pub resets: usize,
}

impl Device for MockDevice {
    fn send_frame(&mut self, frame: &Frame) -> Result<(), DeviceError> {
        self.frames.push(frame.clone());
        Ok(())
    }

    fn reset(&mut self) -> Result<(), DeviceError> {
        self.resets += 1;
        Ok(())
    }
}
