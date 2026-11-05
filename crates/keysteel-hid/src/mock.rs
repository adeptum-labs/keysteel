use crate::{Device, DeviceError};
use keysteel_core::frame::Frame;

/// Records every frame it is sent. Used in tests.
#[derive(Default)]
pub struct MockDevice {
    pub frames: Vec<Frame>,
}

impl Device for MockDevice {
    fn send_frame(&mut self, frame: &Frame) -> Result<(), DeviceError> {
        self.frames.push(frame.clone());
        Ok(())
    }
}
