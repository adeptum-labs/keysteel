mod dbus;
mod hotplug;
mod runtime;

use keysteel_daemon::engine::Engine;
use keysteel_daemon::state;
use keysteel_hid::HidrawDevice;
use runtime::Shared;
use std::sync::Arc;
use zbus::blocking::connection;

fn main() {
    let scene = state::default_path().map(|p| state::load_from(&p)).unwrap_or_else(|| "off".into());
    let device = HidrawDevice::open().ok();
    let shared = Arc::new(Shared::new(Engine::new(device, &scene)));

    let _connection = match connection::Builder::session()
        .and_then(|b| b.name(dbus::BUS_NAME))
        .and_then(|b| b.serve_at(dbus::OBJECT_PATH, dbus::Service { shared: shared.clone() }))
        .and_then(|b| b.build())
    {
        Ok(connection) => connection,
        Err(e) => {
            eprintln!("cannot start D-Bus service: {e}");
            std::process::exit(1);
        }
    };

    let watcher = shared.clone();
    std::thread::spawn(move || {
        hotplug::try_attach(&watcher);
        if let Err(e) = hotplug::watch(watcher) {
            eprintln!("hotplug watcher stopped: {e}");
        }
    });

    shared.render_loop();
}
