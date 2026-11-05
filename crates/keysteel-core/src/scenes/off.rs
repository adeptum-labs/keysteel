use crate::frame::Frame;
use crate::scene::Scene;
use std::time::Duration;

/// Shows nothing.
pub struct Off;

impl Scene for Off {
    fn id(&self) -> &'static str {
        "off"
    }

    fn name(&self) -> &'static str {
        "Off"
    }

    fn animated(&self) -> bool {
        false
    }

    fn shows_picture(&self) -> bool {
        false
    }

    fn render(&mut self, _elapsed: Duration, frame: &mut Frame) {
        frame.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::Scene;
    use std::time::Duration;

    #[test]
    fn off_renders_blank() {
        let mut frame = Frame::new();
        frame.set_pixel(3, 3, true);
        Off.render(Duration::from_secs(1), &mut frame);
        assert_eq!(frame, Frame::new());
    }
}
