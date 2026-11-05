use crate::runtime::Shared;
use keysteel_hid::{is_keyboard_hid_id, HidrawDevice};
use std::os::unix::io::AsRawFd;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

/// Device nodes can take a moment to become accessible after they appear.
const OPEN_ATTEMPTS: u32 = 10;
const OPEN_RETRY: Duration = Duration::from_millis(200);

/// Opens the keyboard and gives it to the engine, replacing any handle it
/// already holds. A handle can go stale without a failed send (unplugged
/// while no frames were being sent), so a node appearing always wins.
pub fn try_attach(shared: &Shared) {
    for _ in 0..OPEN_ATTEMPTS {
        if let Ok(device) = HidrawDevice::open() {
            shared.lock().engine.set_device(Some(device));
            shared.wake();
            return;
        }
        sleep(OPEN_RETRY);
    }
}

/// True when a hidraw node of the keyboard itself appeared, not one of some
/// other device. The kernel names the device in the `HID_ID` of its parent.
fn is_keyboard_added(event: &udev::Event) -> bool {
    event.event_type() == udev::EventType::Add
        && event
            .parent_with_subsystem("hid")
            .ok()
            .flatten()
            .and_then(|hid| hid.property_value("HID_ID").and_then(|v| v.to_str()).map(is_keyboard_hid_id))
            .unwrap_or(false)
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
        let added = monitor.iter().any(|event| is_keyboard_added(&event));
        if added {
            try_attach(&shared);
        }
    }
}
