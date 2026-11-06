mod dbus;
mod hotplug;
mod runtime;
mod scroll;

use keysteel_daemon::engine::Engine;
use keysteel_daemon::state;
use keysteel_hid::HidrawDevice;
use runtime::Shared;
use scroll::Roller;
use std::sync::Arc;
use zbus::blocking::connection;
use zbus::fdo::{RequestNameFlags, RequestNameReply};

fn main() {
    let scene = state::default_path().map(|p| state::load_from(&p)).unwrap_or_else(|| "off".into());
    // The keyboard is attached only after the bus name is ours, so a second
    // copy started by hand fails before it can touch the display.
    let shared = Arc::new(Shared::new(Engine::<HidrawDevice>::new(None, &scene)));

    let roller = Arc::new(Roller::new(
        state::roller_path().map(|p| state::load_roller(&p)).unwrap_or(false),
        state::speed_path().map(|p| state::load_speed(&p)).unwrap_or(state::DEFAULT_SPEED),
    ));

    let connection = match connection::Builder::session()
        .and_then(|b| b.serve_at(dbus::OBJECT_PATH, dbus::Service { shared: shared.clone(), roller: roller.clone() }))
        .and_then(|b| b.build())
    {
        Ok(connection) => connection,
        Err(e) => {
            eprintln!("cannot start D-Bus service: {e}");
            std::process::exit(1);
        }
    };
    // Without DoNotQueue a second copy would wait for the name and run
    // beside the first one, both drawing on the keyboard.
    match connection.request_name_with_flags(dbus::BUS_NAME, RequestNameFlags::DoNotQueue.into()) {
        Ok(RequestNameReply::PrimaryOwner | RequestNameReply::AlreadyOwner) => {}
        Ok(_) | Err(zbus::Error::NameTaken) => {
            eprintln!("{} is already running", dbus::BUS_NAME);
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("cannot take the D-Bus name: {e}");
            std::process::exit(1);
        }
    }

    let watcher = shared.clone();
    std::thread::spawn(move || {
        hotplug::try_attach(&watcher);
        if let Err(e) = hotplug::watch(watcher) {
            // Without the watcher a replugged keyboard is never found again,
            // so stop and let the service manager start a fresh daemon.
            eprintln!("hotplug watcher stopped: {e}");
            std::process::exit(1);
        }
    });

    std::thread::spawn(move || scroll::run(&roller));

    shared.render_loop();
}
