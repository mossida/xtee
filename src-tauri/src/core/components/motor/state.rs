use std::sync::Arc;

use ractor::{ActorProcessingErr, ActorRef, concurrency::JoinHandle};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::core::{
    components::{controller::ControllerMessage, master::MasterMessage},
    protocol::Packet,
    store::Store,
};

use super::{MotorMovement, MotorsLimits};

#[derive(Debug, Clone, Type, Serialize, Deserialize)]
#[serde(tag = "status", content = "data")]
#[serde(rename_all = "kebab-case")]
pub enum MotorStatus {
    Idle,
    Stopping,
    Spinning { position: i32, remaining: u32 },
}

pub struct MotorState {
    pub status: MotorStatus,
    pub max_speed: u32,
    pub config: MotorsLimits,
    pub updates_handle: Option<JoinHandle<()>>,
    pub master: ActorRef<MasterMessage>,
    pub controller: ActorRef<ControllerMessage>,
    pub store: Arc<Store>,
}

impl MotorState {
    pub fn keep(
        &mut self,
        slave: u8,
        movement: &mut MotorMovement,
    ) -> Result<(), ActorProcessingErr> {
        movement.clamp(&self.config);

        self.controller
            .send_message(ControllerMessage::Forward(Packet::MotorSetOutputs {
                slave,
                outputs: true,
            }))?;

        self.controller
            .send_message(ControllerMessage::Forward(Packet::MotorSetSpeed {
                slave,
                speed: movement.speed,
                apply: false,
            }))?;

        self.controller
            .send_message(ControllerMessage::Forward(Packet::MotorSetAcceleration {
                slave,
                acceleration: self.config.acceleration,
            }))?;

        self.controller
            .send_message(ControllerMessage::Forward(Packet::MotorKeep {
                slave,
                direction: movement.direction,
            }))?;

        Ok(())
    }

    pub fn spin(
        &mut self,
        slave: u8,
        movement: &mut MotorMovement,
    ) -> Result<(), ActorProcessingErr> {
        movement.clamp(&self.config);

        // To be sure we make X rotations, we need to stop the motor first and reset the position
        self.controller
            .send_message(ControllerMessage::Forward(Packet::MotorStop {
                slave,
                gentle: false,
            }))?;

        // Enable the motor outputs
        self.controller
            .send_message(ControllerMessage::Forward(Packet::MotorSetOutputs {
                slave,
                outputs: true,
            }))?;

        self.controller
            .send_message(ControllerMessage::Forward(Packet::MotorSetSpeed {
                slave,
                speed: movement.speed,
                apply: false,
            }))?;

        // TODO: Understand why we need to set the acceleration here
        self.controller
            .send_message(ControllerMessage::Forward(Packet::MotorSetAcceleration {
                slave,
                acceleration: self.config.acceleration,
            }))?;

        self.controller
            .send_message(ControllerMessage::Forward(Packet::MotorMove {
                slave,
                direction: movement.direction,
                rotations: movement.rotations,
            }))?;

        Ok(())
    }
}
