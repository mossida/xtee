use std::sync::Arc;

use ractor::{
    async_trait,
    concurrency::{Duration, JoinHandle},
    Actor, ActorProcessingErr, ActorRef, RpcReplyPort,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use tracing::debug;

use crate::{
    components::{controller::ControllerMessage, master::Event},
    error::Error,
    protocol::Packet,
    store::{MotorsLimits, Store, StoreKey},
};

use super::master::MasterMessage;

pub struct Motor {
    pub slave: u8,
}

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
    StartUpdates,
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

pub struct MotorState {
    status: MotorStatus,
    max_speed: u32,
    config: MotorArguments,
    updates_handle: Option<JoinHandle<()>>,
    master: ActorRef<MasterMessage>,
    controller: ActorRef<ControllerMessage>,
}

impl MotorState {
    pub fn keep(
        &mut self,
        slave: u8,
        movement: &mut MotorMovement,
    ) -> Result<(), ActorProcessingErr> {
        movement.clamp(&self.config.limits);

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
                acceleration: self.config.limits.acceleration,
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
        movement.clamp(&self.config.limits);

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
                apply: 0x00,
            }))?;

        // TODO: Understand why we need to set the acceleration here
        self.controller
            .send_message(ControllerMessage::Forward(Packet::MotorSetAcceleration {
                slave,
                acceleration: self.config.limits.acceleration,
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

pub struct MotorArguments {
    pub limits: MotorsLimits,
    store: Arc<Store>,
}

impl MotorArguments {
    pub fn reload(&mut self) -> Result<(), Error> {
        *self = Self::try_from(self.store.clone())?;

        Ok(())
    }
}

impl TryFrom<Arc<Store>> for MotorArguments {
    type Error = Error;

    fn try_from(store: Arc<Store>) -> Result<Self, Error> {
        let limits_value = store.get(StoreKey::MotorsLimits).ok_or(Error::Config)?;
        let limits: MotorsLimits = serde_json::from_value(limits_value)?;

        Ok(Self { limits, store })
    }
}

#[async_trait]
impl Actor for Motor {
    type Msg = MotorMessage;
    type State = MotorState;
    type Arguments = MotorArguments;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        config: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let controller = myself.try_get_supervisor().ok_or(Error::Config)?;
        let master = controller.try_get_supervisor().ok_or(Error::Config)?;

        Ok(MotorState {
            status: MotorStatus::Idle,
            max_speed: 0,
            config,
            updates_handle: None,
            controller: controller.into(),
            master: master.into(),
        })
    }

    async fn post_start(
        &self,
        myself: ActorRef<Self::Msg>,
        _state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        myself.send_message(MotorMessage::StartUpdates)?;

        Ok(())
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        msg: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        let slave = self.slave;

        match msg {
            MotorMessage::StartUpdates => {
                state.updates_handle = Some(
                    state
                        .controller
                        .send_interval(Duration::from_millis(500), move || {
                            ControllerMessage::Forward(Packet::MotorAskStatus { slave })
                        }),
                );
            }
            MotorMessage::GracefulStop => {
                state
                    .controller
                    .send_message(ControllerMessage::Forward(Packet::MotorStop {
                        slave,
                        gentle: true,
                    }))?;
            }
            MotorMessage::EmergencyStop => {
                state
                    .controller
                    .send_message(ControllerMessage::Forward(Packet::MotorStop {
                        slave,
                        gentle: false,
                    }))?;
            }
            MotorMessage::Keep(mut movement) => {
                debug!("Keeping motor {} with {:?}", self.slave, movement);

                state.keep(self.slave, &mut movement)?;
            }
            MotorMessage::Spin(mut movement) => {
                debug!("Spinning motor {} with {:?}", self.slave, movement);

                state.spin(self.slave, &mut movement)?;
            }
            MotorMessage::SetOutputs(outputs) => {
                state.controller.send_message(ControllerMessage::Forward(
                    Packet::MotorSetOutputs {
                        slave: self.slave,
                        outputs,
                    },
                ))?;
            }
            MotorMessage::GetMaxSpeed(reply) => {
                reply.send(state.max_speed)?;
            }
            MotorMessage::ReloadSettings => {
                state.config.reload()?;
            }
            MotorMessage::Packet(packet) => match packet {
                Packet::MotorStatus {
                    slave,
                    running,
                    stopping,
                    position,
                    remaining,
                    ..
                } if slave == self.slave => {
                    state.status = match running {
                        false => MotorStatus::Idle,
                        true if !stopping => MotorStatus::Spinning {
                            position,
                            remaining,
                        },
                        true if stopping => MotorStatus::Stopping,
                        _ => MotorStatus::Idle,
                    };

                    debug!("Motor {} status: {:?}", self.slave, state.status);

                    state
                        .master
                        .send_message(MasterMessage::Event(Event::MotorStatus(
                            self.slave,
                            state.status.clone(),
                        )))?;
                }
                Packet::MotorRecognition { slave, max_speed } if slave == self.slave => {
                    debug!("Motor {} max speed: {}", self.slave, max_speed);

                    state.max_speed = max_speed;
                }
                _ => {}
            },
        }

        Ok(())
    }

    async fn post_stop(
        &self,
        _: ActorRef<Self::Msg>,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        if let Some(handle) = state.updates_handle.take() {
            handle.abort();
        }

        Ok(())
    }
}
