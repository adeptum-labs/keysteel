//! The thread that, while the roller feature is on, grabs the keyboard's
//! media key device and forwards it with the roller turned into a wheel.

use evdev::uinput::VirtualDevice;
use evdev::{AttributeSet, Device, EventType, InputEvent, RelativeAxisCode};
use keysteel_daemon::roller::{map, Mapped};
use std::io;
use std::os::fd::AsRawFd;
use std::sync::{Condvar, Mutex};
use std::time::Duration;

const VENDOR: u16 = 0x1038;
const PRODUCT: u16 = 0x1642;
/// How often the forwarding loop looks up whether the feature was switched off.
const CHECK_EVERY: Duration = Duration::from_millis(200);
const RETRY_AFTER: Duration = Duration::from_secs(2);
/// One notch of a wheel in the high resolution units the kernel uses.
const HI_RES_NOTCH: i32 = 120;

/// The on/off switch shared by the D-Bus service and the worker thread.
pub struct Roller {
    on: Mutex<bool>,
    changed: Condvar,
}

impl Roller {
    pub fn new(on: bool) -> Roller {
        Roller { on: Mutex::new(on), changed: Condvar::new() }
    }

    pub fn get(&self) -> bool {
        *self.on.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn set(&self, on: bool) {
        *self.on.lock().unwrap_or_else(|e| e.into_inner()) = on;
        self.changed.notify_all();
    }

    /// Blocks, using no CPU, until the feature is on.
    fn wait_until_on(&self) {
        let mut on = self.on.lock().unwrap_or_else(|e| e.into_inner());
        while !*on {
            on = self.changed.wait(on).unwrap_or_else(|e| e.into_inner());
        }
    }
}

/// Runs forever: waits for the feature to be on, then forwards until it is
/// switched off or the keyboard goes away, and tries again.
pub fn run(roller: &Roller) {
    let mut last_error = String::new();
    loop {
        roller.wait_until_on();
        match forward(roller) {
            Ok(()) => last_error.clear(),
            Err(e) => {
                // Said once per distinct problem, not every retry.
                if e.to_string() != last_error {
                    eprintln!("roller scrolling: {e}");
                    last_error = e.to_string();
                }
                std::thread::sleep(RETRY_AFTER);
            }
        }
    }
}

fn find_media_keys() -> io::Result<Device> {
    evdev::enumerate()
        .map(|(_, device)| device)
        .find(|device| {
            let id = device.input_id();
            id.vendor() == VENDOR
                && id.product() == PRODUCT
                && device.name().is_some_and(|name| name.ends_with("Consumer Control"))
        })
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "keyboard media key device not found or not readable (is the udev rule installed?)",
            )
        })
}

fn forward(roller: &Roller) -> io::Result<()> {
    let mut source = find_media_keys()?;
    let keys = source
        .supported_keys()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "media key device has no keys"))?;
    let mut axes = AttributeSet::<RelativeAxisCode>::new();
    axes.insert(RelativeAxisCode::REL_WHEEL);
    axes.insert(RelativeAxisCode::REL_WHEEL_HI_RES);
    let mut sink = VirtualDevice::builder()?
        .name("Keysteel roller")
        .with_keys(keys)?
        .with_relative_axes(&axes)?
        .build()?;

    source.set_nonblocking(true)?;
    // The virtual device must exist before the grab, or keys would be lost.
    source.grab()?;
    let fd = source.as_raw_fd();
    while roller.get() {
        let mut poll = libc::pollfd { fd, events: libc::POLLIN, revents: 0 };
        // SAFETY: one valid pollfd, and the count says so.
        let ready = unsafe { libc::poll(&mut poll, 1, CHECK_EVERY.as_millis() as i32) };
        if ready < 0 {
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(error);
        }
        if ready == 0 {
            continue;
        }
        let mut out = Vec::new();
        match source.fetch_events() {
            Ok(events) => {
                for event in events {
                    match map(event.event_type().0, event.code(), event.value()) {
                        Mapped::Pass if event.event_type() == EventType::KEY => out.push(event),
                        Mapped::Pass | Mapped::Swallow => {}
                        Mapped::Wheel(notches) => {
                            let rel = EventType::RELATIVE.0;
                            out.push(InputEvent::new(rel, RelativeAxisCode::REL_WHEEL_HI_RES.0, notches * HI_RES_NOTCH));
                            out.push(InputEvent::new(rel, RelativeAxisCode::REL_WHEEL.0, notches));
                        }
                    }
                }
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => continue,
            Err(e) => return Err(e),
        }
        if !out.is_empty() {
            sink.emit(&out)?;
        }
    }
    // Dropping the source gives the device back to the system.
    Ok(())
}
