use crate::runtime::Shared;
use keysteel_hid::HidrawDevice;
use std::os::unix::io::AsRawFd;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

/// Device nodes can take a moment to become accessible after they appear.
const OPEN_ATTEMPTS: u32 = 10;
const OPEN_RETRY: Duration = Duration::from_millis(200);

/// Attaches the keyboard to the engine if none is attached yet.
pub fn try_attach(shared: &Shared) {
    if shared.lock().engine.device().is_some() {
        return;
    }
    for _ in 0..OPEN_ATTEMPTS {
        if let Ok(device) = HidrawDevice::open() {
            shared.lock().engine.set_device(Some(device));
            shared.wake();
            return;
        }
        sleep(OPEN_RETRY);
    }
}

/// Blocks forever, attaching the keyboard whenever a hidraw node appears.
pub fn watch(shared: Arc<Shared>) -> std::io::Result<()> {
    let monitor = udev::MonitorBuilder::new()?.match_subsystem("hidraw")?.listen()?;
    let mut poll_fd = libc::pollfd { fd: monitor.as_raw_fd(), events: libc::POLLIN, revents: 0 };
    loop {
        // SAFETY: `poll_fd` is a valid pollfd and the count is 1.
        let ready = unsafe { libc::poll(&mut poll_fd, 1, -1) };
        if ready < 0 {
            let error = std::io::Error::last_os_error();
            if error.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            return Err(error);
        }
        let added = monitor
            .iter()
            .any(|event| event.event_type() == udev::EventType::Add);
        if added {
            try_attach(&shared);
        }
    }
}
