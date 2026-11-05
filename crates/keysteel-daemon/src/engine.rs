use keysteel_core::frame::Frame;
use keysteel_core::scene::{scene_by_id, Scene};
use keysteel_hid::Device;
use std::time::Duration;

#[derive(Debug, PartialEq, Eq)]
pub struct UnknownScene(pub String);

impl std::fmt::Display for UnknownScene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unknown scene: {}", self.0)
    }
}

impl std::error::Error for UnknownScene {}

/// Keeps the active scene and shows it on the keyboard, if one is attached.
///
/// `elapsed` values are measured from the moment the current scene started;
/// the caller owns the clock.
pub struct Engine<D: Device> {
    device: Option<D>,
    scene: Box<dyn Scene>,
    last_elapsed: Duration,
    frame: Frame,
}

impl<D: Device> Engine<D> {
    /// Starts on `scene_id` (or Off when it is unknown) and shows it.
    pub fn new(device: Option<D>, scene_id: &str) -> Engine<D> {
        let scene = scene_by_id(scene_id)
            .or_else(|| scene_by_id("off"))
            .expect("the off scene always exists");
        let mut engine = Engine {
            device,
            scene,
            last_elapsed: Duration::ZERO,
            frame: Frame::new(),
        };
        engine.present();
        engine
    }

    pub fn current(&self) -> &str {
        self.scene.id()
    }

    pub fn device(&self) -> Option<&D> {
        self.device.as_ref()
    }

    /// True when the scene changes over time and there is a keyboard to draw on.
    pub fn needs_ticks(&self) -> bool {
        self.scene.animated() && self.device.is_some()
    }

    /// Switches scene and shows its first frame. An unknown id changes nothing.
    pub fn set_scene(&mut self, id: &str) -> Result<(), UnknownScene> {
        let scene = scene_by_id(id).ok_or_else(|| UnknownScene(id.to_string()))?;
        self.scene = scene;
        self.last_elapsed = Duration::ZERO;
        self.present();
        Ok(())
    }

    /// Draws the next frame of an animated scene.
    pub fn tick(&mut self, elapsed: Duration) {
        self.last_elapsed = elapsed;
        if self.scene.animated() {
            self.present();
        }
    }

    /// Attaches or detaches the keyboard and shows the current frame on it.
    pub fn set_device(&mut self, device: Option<D>) {
        self.device = device;
        self.present();
    }

    fn present(&mut self) {
        let Some(device) = self.device.as_mut() else {
            return;
        };
        let result = if self.scene.shows_picture() {
            self.scene.render(self.last_elapsed, &mut self.frame);
            device.send_frame(&self.frame)
        } else {
            device.reset()
        };
        if result.is_err() {
            self.device = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keysteel_core::frame::Frame;
    use keysteel_hid::{Device, DeviceError, MockDevice};
    use std::time::Duration;

    struct FailingDevice;

    impl Device for FailingDevice {
        fn send_frame(&mut self, _frame: &Frame) -> Result<(), DeviceError> {
            Err(DeviceError::Hid("unplugged".into()))
        }

        fn reset(&mut self) -> Result<(), DeviceError> {
            Err(DeviceError::Hid("unplugged".into()))
        }
    }

    fn is_blank(frame: &Frame) -> bool {
        frame.as_bytes().iter().all(|b| *b == 0)
    }

    fn mock_engine(scene: &str) -> Engine<MockDevice> {
        Engine::new(Some(MockDevice::default()), scene)
    }

    #[test]
    fn set_scene_switches_and_sends_frame() {
        let mut engine = mock_engine("off");
        engine.set_scene("boing").unwrap();
        assert_eq!(engine.current(), "boing");
        let frames = &engine.device().unwrap().frames;
        assert_eq!(frames.len(), 1);
        assert!(!is_blank(&frames[0]));
    }

    #[test]
    fn starting_on_off_resets_instead_of_sending_a_frame() {
        let engine = mock_engine("off");
        let device = engine.device().unwrap();
        assert_eq!(device.resets, 1);
        assert!(device.frames.is_empty());
    }

    #[test]
    fn attaching_a_device_on_off_resets_it() {
        let mut engine: Engine<MockDevice> = Engine::new(None, "off");
        engine.set_device(Some(MockDevice::default()));
        let device = engine.device().unwrap();
        assert_eq!(device.resets, 1);
        assert!(device.frames.is_empty());
    }

    #[test]
    fn failed_reset_drops_device() {
        let engine = Engine::new(Some(FailingDevice), "off");
        assert!(engine.device().is_none());
    }

    #[test]
    fn set_unknown_scene_errors_and_keeps_current() {
        let mut engine = mock_engine("boing");
        let before = engine.device().unwrap().frames.len();
        assert!(engine.set_scene("nope").is_err());
        assert_eq!(engine.current(), "boing");
        assert_eq!(engine.device().unwrap().frames.len(), before);
    }

    #[test]
    fn unknown_initial_scene_falls_back_to_off() {
        let engine = mock_engine("nope");
        assert_eq!(engine.current(), "off");
    }

    #[test]
    fn off_scene_resets_once_and_sends_nothing_on_tick() {
        let mut engine = mock_engine("boing");
        let sent = engine.device().unwrap().frames.len();
        engine.set_scene("off").unwrap();
        engine.tick(Duration::from_secs(1));
        engine.tick(Duration::from_secs(2));
        let device = engine.device().unwrap();
        assert_eq!(device.resets, 1);
        assert_eq!(device.frames.len(), sent);
    }

    #[test]
    fn animated_scene_sends_frame_each_tick() {
        let mut engine = mock_engine("boing");
        let sent = engine.device().unwrap().frames.len();
        for ms in [33, 66, 99] {
            engine.tick(Duration::from_millis(ms));
        }
        assert_eq!(engine.device().unwrap().frames.len(), sent + 3);
    }

    #[test]
    fn set_device_resends_current_frame() {
        let mut engine: Engine<MockDevice> = Engine::new(None, "boing");
        engine.set_device(Some(MockDevice::default()));
        let frames = &engine.device().unwrap().frames;
        assert_eq!(frames.len(), 1);
        assert!(!is_blank(&frames[0]));
    }

    #[test]
    fn failed_send_drops_device_until_replaced() {
        let mut engine = Engine::new(Some(FailingDevice), "boing");
        assert!(engine.device().is_none());
        engine.tick(Duration::from_millis(33));
        assert!(engine.device().is_none());
    }

    #[test]
    fn needs_ticks_only_for_animated_scene_with_device() {
        let mut engine = mock_engine("boing");
        assert!(engine.needs_ticks());
        engine.set_scene("off").unwrap();
        assert!(!engine.needs_ticks());
        let engine: Engine<MockDevice> = Engine::new(None, "boing");
        assert!(!engine.needs_ticks());
    }
}
