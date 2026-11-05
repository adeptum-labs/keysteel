mod window;

use gtk4::prelude::*;
use gtk4::{glib, Application};

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id("org.keysteel.App").build();
    app.connect_activate(window::build);
    app.run()
}
