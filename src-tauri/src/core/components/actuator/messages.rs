use serde::{Deserialize, Serialize};
use specta::Type;

use crate::core::protocol::Packet;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Type)]
#[serde(tag = "status", content = "data")]
#[serde(rename_all = "kebab-case")]
pub enum ActuatorStatus {
    Overloaded,
    Unloading,
    Loading { target: f32 },
    Keeping { target: f32 },
    Idle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "kebab-case")]
pub enum ActuatorMovement {
    Load,
    Unload,
}

impl ActuatorMovement {
    pub fn is_unload(self) -> bool {
        self == Self::Unload
    }

    pub fn is_load(self) -> bool {
        self == Self::Load
    }

    pub fn into_packet(self) -> Packet {
        ActuatorDirection::into_packet(self.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "kebab-case")]
pub enum ActuatorDirection {
    Forward,
    Backward,
}

impl ActuatorDirection {
    pub fn unload() -> Self {
        Self::Forward
    }

    pub fn load() -> Self {
        Self::Backward
    }
}

impl ActuatorDirection {
    pub fn is_unload(self) -> bool {
        self == Self::unload()
    }

    #[allow(dead_code)]
    pub fn is_load(self) -> bool {
        self == Self::load()
    }

    pub fn into_packet(self) -> Packet {
        Packet::ActuatorMove {
            direction: self.is_unload(),
        }
    }
}

impl From<ActuatorMovement> for ActuatorDirection {
    fn from(value: ActuatorMovement) -> Self {
        match value {
            ActuatorMovement::Load => Self::load(),
            ActuatorMovement::Unload => Self::unload(),
        }
    }
}

#[derive(Debug)]
pub enum ActuatorMessage {
    Load(f32),
    Keep(f32),
    Move(ActuatorMovement),
    Unload,
    Stop,
    Packet(Packet),
    ReloadSettings,
}

impl From<Packet> for ActuatorMessage {
    fn from(value: Packet) -> Self {
        ActuatorMessage::Packet(value)
    }
}
