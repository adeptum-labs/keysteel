use gtk4::prelude::*;
use gtk4::{
    gdk, glib, Application, ApplicationWindow, Box as GtkBox, CheckButton, Label, Orientation,
    Picture,
};
use keysteel_app::client::Client;
use keysteel_app::preview::{preview_size, scale_frame_to_rgba};
use keysteel_core::frame::Frame;
use keysteel_core::scene::{all_scenes, scene_by_id, Scene};
use std::cell::RefCell;
use std::rc::Rc;
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

pub fn build(app: &Application) {
    let (client, mut status) = match Client::connect() {
        Ok(client) => (Some(client), String::new()),
        Err(error) => (None, error.to_string()),
    };
    let client = Rc::new(client);

    let local_scenes = || -> Vec<(String, String)> {
        all_scenes().iter().map(|s| (s.id().to_string(), s.name().to_string())).collect()
    };
    let scenes = match client.as_ref() {
        Some(client) => client.list_scenes().unwrap_or_else(|e| {
            status = e.to_string();
            local_scenes()
        }),
        None => local_scenes(),
    };
    let current = client
        .as_ref()
        .as_ref()
        .and_then(|c| c.current().ok())
        .unwrap_or_else(|| "off".to_string());

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

    let status_label = Label::new(Some(&status));
    status_label.set_xalign(0.0);

    let list = GtkBox::new(Orientation::Vertical, 6);
    list.append(&Label::builder().label("Display").xalign(0.0).css_classes(["heading"]).build());
    let mut first: Option<CheckButton> = None;
    let mut buttons = Vec::new();
    for (id, name) in &scenes {
        let button = CheckButton::with_label(name);
        if let Some(first) = &first {
            button.set_group(Some(first));
        } else {
            first = Some(button.clone());
        }
        button.set_active(*id == current);
        button.set_sensitive(client.is_some());
        list.append(&button);
        buttons.push((id.clone(), button));
    }
    select_scene(&preview, &current);

    for (id, button) in buttons {
        let preview = preview.clone();
        let client = client.clone();
        let status_label = status_label.clone();
        button.connect_toggled(move |button| {
            if !button.is_active() {
                return;
            }
            select_scene(&preview, &id);
            if let Some(client) = client.as_ref() {
                match client.set_scene(&id) {
                    Ok(()) => status_label.set_text(""),
                    Err(error) => status_label.set_text(&error.to_string()),
                }
            }
        });
    }

    let row = GtkBox::new(Orientation::Horizontal, 24);
    row.append(&list);
    row.append(&picture);
    let content = GtkBox::new(Orientation::Vertical, 12);
    content.set_margin_top(16);
    content.set_margin_bottom(16);
    content.set_margin_start(16);
    content.set_margin_end(16);
    content.append(&row);
    content.append(&status_label);

    ApplicationWindow::builder()
        .application(app)
        .title("Keysteel")
        .child(&content)
        .build()
        .present();
}
