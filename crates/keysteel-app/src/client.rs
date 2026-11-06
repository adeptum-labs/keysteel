use zbus::blocking::{Connection, Proxy};
use zbus::fdo;

const BUS_NAME: &str = "org.keysteel.Daemon";
const OBJECT_PATH: &str = "/org/keysteel/Daemon";

#[derive(Debug)]
pub enum ClientError {
    /// Nobody owns the daemon's bus name and it could not be started.
    DaemonNotRunning,
    Call(String),
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::DaemonNotRunning => write!(f, "daemon not running"),
            ClientError::Call(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for ClientError {}

fn map_error(error: zbus::Error) -> ClientError {
    match fdo::Error::from(error) {
        fdo::Error::ServiceUnknown(_)
        | fdo::Error::NameHasNoOwner(_)
        | fdo::Error::SpawnExecFailed(_)
        | fdo::Error::SpawnForkFailed(_)
        | fdo::Error::SpawnChildExited(_)
        | fdo::Error::SpawnChildSignaled(_)
        | fdo::Error::SpawnFailed(_)
        | fdo::Error::SpawnFailedToSetup(_)
        | fdo::Error::SpawnConfigInvalid(_)
        | fdo::Error::SpawnServiceNotValid(_)
        | fdo::Error::SpawnServiceNotFound(_)
        | fdo::Error::SpawnPermissionsInvalid(_)
        | fdo::Error::SpawnFileInvalid(_)
        | fdo::Error::SpawnNoMemory(_) => ClientError::DaemonNotRunning,
        other => ClientError::Call(other.to_string()),
    }
}

/// Talks to the daemon over the session bus.
pub struct Client {
    proxy: Proxy<'static>,
}

impl Client {
    /// Connects to the session bus and checks the daemon answers.
    pub fn connect() -> Result<Client, ClientError> {
        let connection = Connection::session().map_err(map_error)?;
        let proxy = Proxy::new(&connection, BUS_NAME, OBJECT_PATH, BUS_NAME).map_err(map_error)?;
        let client = Client { proxy };
        client.current()?;
        Ok(client)
    }

    /// All scenes as (id, display name).
    pub fn list_scenes(&self) -> Result<Vec<(String, String)>, ClientError> {
        self.proxy.call("ListScenes", &()).map_err(map_error)
    }

    pub fn current(&self) -> Result<String, ClientError> {
        self.proxy.call("GetScene", &()).map_err(map_error)
    }

    pub fn set_scene(&self, id: &str) -> Result<(), ClientError> {
        self.proxy.call("SetScene", &(id,)).map_err(map_error)
    }

    /// Whether the roller scrolls the window instead of changing the volume.
    pub fn roller_scroll(&self) -> Result<bool, ClientError> {
        self.proxy.call("GetRollerScroll", &()).map_err(map_error)
    }

    /// Wheel notches for each click of the roller.
    pub fn roller_speed(&self) -> Result<u32, ClientError> {
        self.proxy.call("GetRollerSpeed", &()).map_err(map_error)
    }

    pub fn set_roller_speed(&self, speed: u32) -> Result<(), ClientError> {
        self.proxy.call("SetRollerSpeed", &(speed,)).map_err(map_error)
    }

    pub fn set_roller_scroll(&self, on: bool) -> Result<(), ClientError> {
        self.proxy.call("SetRollerScroll", &(on,)).map_err(map_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zbus::fdo;

    fn fdo_error(error: fdo::Error) -> zbus::Error {
        zbus::Error::FDO(Box::new(error))
    }

    #[test]
    fn missing_service_maps_to_daemon_not_running() {
        let error = map_error(fdo_error(fdo::Error::ServiceUnknown("gone".into())));
        assert!(matches!(error, ClientError::DaemonNotRunning));
    }

    #[test]
    fn name_without_owner_maps_to_daemon_not_running() {
        let error = map_error(fdo_error(fdo::Error::NameHasNoOwner("gone".into())));
        assert!(matches!(error, ClientError::DaemonNotRunning));
    }

    #[test]
    fn activation_failures_map_to_daemon_not_running() {
        let errors = vec![
            fdo::Error::SpawnExecFailed("x".into()),
            fdo::Error::SpawnChildExited("x".into()),
            fdo::Error::SpawnChildSignaled("x".into()),
            fdo::Error::SpawnFailed("x".into()),
            fdo::Error::SpawnServiceNotFound("x".into()),
            fdo::Error::SpawnServiceNotValid("x".into()),
        ];
        for error in errors {
            let text = error.to_string();
            let mapped = map_error(fdo_error(error));
            assert!(matches!(mapped, ClientError::DaemonNotRunning), "{text}");
        }
    }

    #[test]
    fn other_errors_keep_their_message() {
        let error = map_error(fdo_error(fdo::Error::InvalidArgs("unknown scene: x".into())));
        match error {
            ClientError::Call(message) => assert!(message.contains("unknown scene: x")),
            other => panic!("unexpected {other:?}"),
        }
    }
}
