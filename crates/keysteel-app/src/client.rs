// SPDX-License-Identifier: GPL-3.0-or-later
//
// Copyright © 2026 Adeptum AB, Org.nr 559494-1824, and Adam Waldenberg
//
// This file is part of Keysteel.
//
// Keysteel is free software: you can redistribute it and/or modify it under
// the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option)
// any later version.
//
// Keysteel is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
// FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for
// more details.
//
// You should have received a copy of the GNU General Public License along
// with Keysteel. If not, see <https://www.gnu.org/licenses/>.

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

/// A lighting scheme and which settings it uses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LightScheme {
    pub id: String,
    pub name: String,
    pub takes_colour: bool,
    pub uses_speed: bool,
    pub uses_intensity: bool,
}

/// The lighting the daemon applies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lighting {
    pub scheme: String,
    /// Percent, 0 to 100.
    pub intensity: u8,
    pub colour: (u8, u8, u8),
    /// 1 to 10.
    pub speed: u8,
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

    /// All lighting schemes.
    pub fn list_light_schemes(&self) -> Result<Vec<LightScheme>, ClientError> {
        let rows: Vec<(String, String, bool, bool, bool)> =
            self.proxy.call("ListLightSchemes", &()).map_err(map_error)?;
        Ok(rows
            .into_iter()
            .map(|(id, name, takes_colour, uses_speed, uses_intensity)| LightScheme {
                id,
                name,
                takes_colour,
                uses_speed,
                uses_intensity,
            })
            .collect())
    }

    /// The current lighting.
    pub fn lighting(&self) -> Result<Lighting, ClientError> {
        let (scheme, intensity, red, green, blue, speed): (String, u8, u8, u8, u8, u8) =
            self.proxy.call("GetLighting", &()).map_err(map_error)?;
        Ok(Lighting { scheme, intensity, colour: (red, green, blue), speed })
    }

    pub fn set_lighting(&self, lighting: &Lighting) -> Result<(), ClientError> {
        let (red, green, blue) = lighting.colour;
        self.proxy
            .call("SetLighting", &(lighting.scheme.as_str(), lighting.intensity, red, green, blue, lighting.speed))
            .map_err(map_error)
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
