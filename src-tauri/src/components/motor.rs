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

use super::{master::MasterMessage, Component, Handler, SpawnArgs};

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
    config: MotorsLimits,
    updates_handle: Option<JoinHandle<()>>,
    master: ActorRef<MasterMessage>,
    controller: ActorRef<ControllerMessage>,
    store: Arc<Store>,
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

impl Component for Motor {
    async fn spawn(self, args: SpawnArgs) -> Result<Handler<MotorMessage>, ActorProcessingErr> {
        let name = format!("motor-{}", self.slave);
        let controller = args.controller.get_cell();

        let (cell, _) = Motor::spawn_linked(Some(name), self, args, controller).await?;

        Ok(Handler { cell })
    }
}

#[async_trait]
impl Actor for Motor {
    type Msg = MotorMessage;
    type State = MotorState;
    type Arguments = SpawnArgs;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        SpawnArgs { store, controller }: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let config = MotorsLimits::try_from(store.clone())?;
        let master = controller.try_get_supervisor().ok_or(Error::Config)?;

        Ok(MotorState {
            status: MotorStatus::Idle,
            max_speed: 0,
            config,
            updates_handle: None,
            controller,
            master: master.into(),
            store,
        })
    }

    async fn post_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        let slave = self.slave;
        state.updates_handle = Some(
            state
                .controller
                .send_interval(Duration::from_millis(500), move || {
                    ControllerMessage::Forward(Packet::MotorAskStatus { slave })
                }),
        );

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
                state.config = MotorsLimits::try_from(state.store.clone())?;
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
