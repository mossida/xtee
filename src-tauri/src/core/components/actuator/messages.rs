use serde::{Deserialize, Serialize};
use specta::Type;

use crate::core::protocol::Packet;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Type)]
#[serde(tag = "status", content = "data")]
#[serde(rename_all = "kebab-case")]
pub enum ActuatorStatus {
    Loading { target: f32 },
    Keeping { target: f32 },
    Idle,
}

pub enum ActuatorMessage {
    Load(f32),
    Keep(f32),
    Move(bool),
    Stop,
    Packet(Packet),
    ReloadSettings,
}

impl From<Packet> for ActuatorMessage {
    fn from(value: Packet) -> Self {
        ActuatorMessage::Packet(value)
    }
}
