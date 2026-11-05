use crate::runtime::Shared;
use keysteel_core::scene::all_scenes;
use keysteel_daemon::state;
use std::sync::Arc;
use std::time::Instant;
use zbus::{fdo, interface};

pub const BUS_NAME: &str = "org.keysteel.Daemon";
pub const OBJECT_PATH: &str = "/org/keysteel/Daemon";

pub struct Service {
    pub shared: Arc<Shared>,
}

#[interface(name = "org.keysteel.Daemon")]
impl Service {
    /// All scenes as (id, display name).
    fn list_scenes(&self) -> Vec<(String, String)> {
        all_scenes()
            .iter()
            .map(|scene| (scene.id().to_string(), scene.name().to_string()))
            .collect()
    }

    fn set_scene(&self, id: &str) -> fdo::Result<()> {
        {
            let mut inner = self.shared.lock();
            inner
                .engine
                .set_scene(id)
                .map_err(|e| fdo::Error::InvalidArgs(e.to_string()))?;
            inner.started = Instant::now();
        }
        self.shared.wake();
        if let Some(path) = state::default_path() {
            if let Err(e) = state::save_to(&path, id) {
                eprintln!("cannot save scene: {e}");
            }
        }
        Ok(())
    }

    fn get_scene(&self) -> String {
        self.shared.lock().engine.current().to_string()
    }
}
