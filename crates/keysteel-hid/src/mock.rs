use crate::{Device, DeviceError, KeyColour, KeyLights};
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

/// Records every lighting change. Used in tests.
#[derive(Default)]
pub struct MockLights {
    pub sets: Vec<Vec<KeyColour>>,
    pub clears: usize,
}

impl KeyLights for MockLights {
    fn set_keys(&mut self, keys: &[KeyColour]) -> Result<(), DeviceError> {
        self.sets.push(keys.to_vec());
        Ok(())
    }

    fn clear_keys(&mut self) -> Result<(), DeviceError> {
        self.clears += 1;
        Ok(())
    }
}
