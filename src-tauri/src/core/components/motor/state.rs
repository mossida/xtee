use ractor::{concurrency::JoinHandle, ActorProcessingErr, ActorRef};
use std::sync::Arc;

use crate::{
    core::{
        components::{controller::ControllerMessage, master::MasterMessage},
        protocol::Packet,
        store::{MotorsLimits, Store, StoreKey},
    },
    utils::error::Error,
};

use super::messages::{MotorMovement, MotorStatus};

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
                apply: 0x00,
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

        self.controller
            .send_message(ControllerMessage::Forward(Packet::MotorStop {
                slave,
                gentle: false,
            }))?;

        self.controller
            .send_message(ControllerMessage::Forward(Packet::MotorSetOutputs {
                slave,
                outputs: true,
            }))?;

        self.controller
            .send_message(ControllerMessage::Forward(Packet::MotorSetSpeed {
                slave,
                speed: movement.speed,
                apply: 0x00,
            }))?;

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

impl TryFrom<Arc<Store>> for MotorsLimits {
    type Error = Error;

    fn try_from(store: Arc<Store>) -> Result<Self, Error> {
        let limits_value = store.get(StoreKey::MotorsLimits).ok_or(Error::Config)?;
        let limits: MotorsLimits = serde_json::from_value(limits_value)?;

        Ok(limits)
    }
}
