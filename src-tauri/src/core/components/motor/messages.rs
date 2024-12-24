use ractor::RpcReplyPort;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::core::{protocol::Packet, store::MotorsLimits};

#[derive(Debug, Clone, Type, Serialize, Deserialize)]
pub struct MotorMovement {
    pub speed: u32,
    pub direction: bool,
    pub rotations: u16,
}

impl MotorMovement {
    pub fn clamp(&mut self, limits: &MotorsLimits) {
        self.speed = self.speed.clamp(1, limits.max_speed);
        self.rotations = self.rotations.clamp(1, limits.max_rotations as u16);
    }
}

#[derive(Debug)]
pub enum MotorMessage {
    Keep(MotorMovement),
    Spin(MotorMovement),
    GracefulStop,
    EmergencyStop,
    Packet(Packet),
    SetOutputs(bool),
    GetMaxSpeed(RpcReplyPort<u32>),
    ReloadSettings,
}

impl From<Packet> for MotorMessage {
    fn from(packet: Packet) -> Self {
        MotorMessage::Packet(packet)
    }
}

#[derive(Debug, Clone, Type, Serialize, Deserialize)]
#[serde(tag = "status", content = "data")]
#[serde(rename_all = "kebab-case")]
pub enum MotorStatus {
    Idle,
    Stopping,
    Spinning { position: i32, remaining: u32 },
}
