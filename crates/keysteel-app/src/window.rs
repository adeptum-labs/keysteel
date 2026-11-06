use gtk4::prelude::*;
use gtk4::{
    gdk, gio, glib, Application, ApplicationWindow, Box as GtkBox, Button, CheckButton, Label,
    Orientation, Picture,
};
use keysteel_app::client::{Client, ClientError};
use keysteel_app::preview::{preview_size, scale_frame_to_rgba};
use keysteel_core::frame::Frame;
use keysteel_core::scene::{all_scenes, scene_by_id, Scene};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

const SCALE: usize = 3;
const FRAME_INTERVAL: Duration = Duration::from_millis(33);

/// The picture beside the scene list: the selected scene, drawn locally.
struct Preview {
    picture: Picture,
    scene: Box<dyn Scene>,
    started: Instant,
    frame: Frame,
    ticking: bool,
}

impl Preview {
    fn draw(&mut self) {
        self.scene.render(self.started.elapsed(), &mut self.frame);
        let (width, height) = preview_size(SCALE);
        let bytes = glib::Bytes::from_owned(scale_frame_to_rgba(&self.frame, SCALE));
        let texture = gdk::MemoryTexture::new(
            width as i32,
            height as i32,
            gdk::MemoryFormat::R8g8b8a8,
            &bytes,
            width * 4,
        );
        self.picture.set_paintable(Some(&texture));
    }
}

fn select_scene(preview: &Rc<RefCell<Preview>>, id: &str) {
    let Some(scene) = scene_by_id(id) else {
        return;
    };
    let start_timer = {
        let mut p = preview.borrow_mut();
        p.scene = scene;
        p.started = Instant::now();
        p.draw();
        let start = p.scene.animated() && !p.ticking;
        p.ticking |= start;
        start
    };
    if start_timer {
        let preview = preview.clone();
        glib::timeout_add_local(FRAME_INTERVAL, move || {
            let mut p = preview.borrow_mut();
            p.draw();
            if p.scene.animated() {
                glib::ControlFlow::Continue
            } else {
                p.ticking = false;
                glib::ControlFlow::Break
            }
        });
    }
}

/// What the daemon told us when we connected.
struct Connected {
    client: Client,
    scenes: Vec<(String, String)>,
    current: String,
    roller: bool,
}

fn connect_blocking() -> Result<Connected, ClientError> {
    let client = Client::connect()?;
    let scenes = client.list_scenes()?;
    let current = client.current()?;
    let roller = client.roller_scroll()?;
    Ok(Connected { client, scenes, current, roller })
}

fn local_scenes() -> Vec<(String, String)> {
    all_scenes().iter().map(|s| (s.id().to_string(), s.name().to_string())).collect()
}

/// The widgets that change when the connection state changes.
struct Ui {
    radios: GtkBox,
    status: Label,
    retry: Button,
    preview: Rc<RefCell<Preview>>,
    roller: CheckButton,
    /// The daemon connection, once there is one.
    client: RefCell<Option<Arc<Client>>>,
    /// True while the tick is set from the daemon's answer, so that does not
    /// count as the user clicking it.
    syncing: Cell<bool>,
}

/// Shows the daemon's roller setting, or disables the tick without one.
fn show_roller(ui: &Ui, client: Option<Arc<Client>>, on: bool) {
    ui.syncing.set(true);
    ui.roller.set_active(on);
    ui.roller.set_sensitive(client.is_some());
    ui.syncing.set(false);
    *ui.client.borrow_mut() = client;
}

/// Fills the scene list. Without a client the list is shown but disabled.
fn populate(ui: &Rc<Ui>, client: Option<Arc<Client>>, scenes: &[(String, String)], current: &str) {
    while let Some(child) = ui.radios.first_child() {
        ui.radios.remove(&child);
    }
    let mut first: Option<CheckButton> = None;
    let mut buttons = Vec::new();
    for (id, name) in scenes {
        let button = CheckButton::with_label(name);
        if let Some(first) = &first {
            button.set_group(Some(first));
        } else {
            first = Some(button.clone());
        }
        button.set_active(id == current);
        button.set_sensitive(client.is_some());
        ui.radios.append(&button);
        buttons.push((id.clone(), button));
    }
    select_scene(&ui.preview, current);

    // Connected only after the initial choice, so setting it does not call the daemon.
    for (id, button) in buttons {
        let ui = ui.clone();
        let client = client.clone();
        button.connect_toggled(move |button| {
            if !button.is_active() {
                return;
            }
            select_scene(&ui.preview, &id);
            let Some(client) = client.clone() else {
                return;
            };
            let id = id.clone();
            let status = ui.status.clone();
            glib::spawn_future_local(async move {
                match gio::spawn_blocking(move || client.set_scene(&id)).await {
                    Ok(Ok(())) => status.set_text(""),
                    Ok(Err(error)) => status.set_text(&error.to_string()),
                    Err(_) => status.set_text("scene change failed"),
                }
            });
        });
    }
}

/// Connects to the daemon without blocking the window, then fills the list.
fn connect(ui: Rc<Ui>) {
    ui.retry.set_visible(false);
    ui.status.set_text("Connecting...");
    glib::spawn_future_local(async move {
        match gio::spawn_blocking(connect_blocking).await {
            Ok(Ok(connected)) => {
                ui.status.set_text("");
                let client = Some(Arc::new(connected.client));
                show_roller(&ui, client.clone(), connected.roller);
                populate(&ui, client, &connected.scenes, &connected.current);
            }
            failure => {
                let message = match failure {
                    Ok(Err(error)) => error.to_string(),
                    _ => "could not reach the daemon".to_string(),
                };
                ui.status.set_text(&message);
                show_roller(&ui, None, false);
                populate(&ui, None, &local_scenes(), "off");
                ui.retry.set_visible(true);
            }
        }
    });
}

pub fn build(app: &Application) {
    if let Some(window) = app.windows().first() {
        window.present();
        return;
    }

    let (width, height) = preview_size(SCALE);
    let picture = Picture::new();
    picture.set_size_request(width as i32, height as i32);
    picture.set_can_shrink(false);
    let preview = Rc::new(RefCell::new(Preview {
        picture: picture.clone(),
        scene: scene_by_id("off").expect("the off scene always exists"),
        started: Instant::now(),
        frame: Frame::new(),
        ticking: false,
    }));

    let radios = GtkBox::new(Orientation::Vertical, 6);
    let list = GtkBox::new(Orientation::Vertical, 6);
    list.append(&Label::builder().label("Display").xalign(0.0).css_classes(["heading"]).build());
    list.append(&radios);

    let status = Label::builder().xalign(0.0).hexpand(true).build();
    let retry = Button::with_label("Retry");
    retry.set_visible(false);
    let status_row = GtkBox::new(Orientation::Horizontal, 12);
    status_row.append(&status);
    status_row.append(&retry);

    let roller = CheckButton::with_label("Roller scrolls the window");
    roller.set_tooltip_text(Some("Turn the roller into a mouse wheel instead of changing the volume"));
    roller.set_sensitive(false);
    list.append(&roller);

    let row = GtkBox::new(Orientation::Horizontal, 24);
    row.append(&list);
    row.append(&picture);
    let content = GtkBox::new(Orientation::Vertical, 12);
    content.set_margin_top(16);
    content.set_margin_bottom(16);
    content.set_margin_start(16);
    content.set_margin_end(16);
    content.append(&row);
    content.append(&status_row);

    ApplicationWindow::builder()
        .application(app)
        .title("Keysteel")
        .child(&content)
        .build()
        .present();

    let ui = Rc::new(Ui {
        radios,
        status,
        retry: retry.clone(),
        preview,
        roller: roller.clone(),
        client: RefCell::new(None),
        syncing: Cell::new(false),
    });
    {
        let ui = ui.clone();
        roller.connect_toggled(move |button| {
            if ui.syncing.get() {
                return;
            }
            let Some(client) = ui.client.borrow().clone() else {
                return;
            };
            let on = button.is_active();
            let status = ui.status.clone();
            glib::spawn_future_local(async move {
                match gio::spawn_blocking(move || client.set_roller_scroll(on)).await {
                    Ok(Ok(())) => status.set_text(""),
                    Ok(Err(error)) => status.set_text(&error.to_string()),
                    Err(_) => status.set_text("roller setting failed"),
                }
            });
        });
    }
    {
        let ui = ui.clone();
        retry.connect_clicked(move |_| connect(ui.clone()));
    }
    connect(ui);
}
