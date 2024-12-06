use ractor::{
    async_trait,
    concurrency::{Duration, JoinHandle},
    rpc, Actor, ActorCell, ActorProcessingErr, ActorRef, RpcReplyPort,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use tracing::debug;

use crate::{components::controller::ControllerMessage, error::ControllerError, protocol::Packet};

use super::mux::MuxMessage;

pub struct Motor {
    pub slave: u8,
}

#[derive(Debug, Clone, Type, Serialize, Deserialize)]
pub struct MotorMovement {
    pub speed: u32,
    pub direction: u8,
    pub rotations: u16,
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
}

impl From<Packet> for MotorMessage {
    fn from(packet: Packet) -> Self {
        MotorMessage::Packet(packet)
    }
}

#[derive(Debug, Clone)]
pub enum MotorStatus {
    Idle,
    Spinning { position: i32, remaining: u32 },
}

#[derive(Debug)]
pub struct MotorState {
    status: MotorStatus,
    max_speed: u32,
    updates_handle: Option<JoinHandle<()>>,
    mux: Option<ActorRef<MuxMessage>>,
}

impl MotorState {
    pub fn keep(&mut self, slave: u8, movement: MotorMovement) -> Result<(), ActorProcessingErr> {
        let mux = self.mux.as_ref().ok_or(ControllerError::MissingMux)?;

        mux.send_message(MuxMessage::Write(Packet::MotorSetOutputs {
            slave,
            outputs: 0x01,
        }))?;

        mux.send_message(MuxMessage::Write(Packet::MotorSetSpeed {
            slave,
            speed: movement.speed,
            apply: 0x01,
        }))?;

        mux.send_message(MuxMessage::Write(Packet::MotorSetAcceleration {
            slave,
            acceleration: 1000,
        }))?;

        mux.send_message(MuxMessage::Write(Packet::MotorKeep {
            slave,
            direction: movement.direction,
        }))?;

        Ok(())
    }

    pub fn spin(&mut self, slave: u8, movement: MotorMovement) -> Result<(), ActorProcessingErr> {
        let mux = self.mux.as_ref().ok_or(ControllerError::MissingMux)?;

        // To be sure we make X rotations, we need to stop the motor first and reset the position
        mux.send_message(MuxMessage::Write(Packet::MotorStop { slave, mode: 0x00 }))?;

        // Enable the motor outputs
        mux.send_message(MuxMessage::Write(Packet::MotorSetOutputs {
            slave,
            outputs: 0x01,
        }))?;

        mux.send_message(MuxMessage::Write(Packet::MotorSetSpeed {
            slave,
            speed: movement.speed,
            apply: 0x01,
        }))?;

        // TODO: Understand why we need to set the acceleration here
        mux.send_message(MuxMessage::Write(Packet::MotorSetAcceleration {
            slave,
            acceleration: 1000,
        }))?;

        mux.send_message(MuxMessage::Write(Packet::MotorMove {
            slave,
            direction: movement.direction,
            rotations: movement.rotations,
        }))?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MotorArguments {}

#[async_trait]
impl Actor for Motor {
    type Msg = MotorMessage;
    type State = MotorState;
    type Arguments = MotorArguments;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        _config: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        Ok(MotorState {
            status: MotorStatus::Idle,
            max_speed: 0,
            updates_handle: None,
            mux: None,
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
        myself: ActorRef<Self::Msg>,
        msg: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        if state.mux.is_none() {
            let controller = myself
                .try_get_superivisor()
                .ok_or(ControllerError::ConfigError)?;

            let mux = rpc::call(&controller, ControllerMessage::FetchMux, None)
                .await?
                .success_or(ControllerError::MissingMux)?;

            debug!("Motor got mux: {:?}", mux.get_name());

            state.mux = Some(mux);
        }

        let slave = self.slave;
        let mux = state.mux.as_ref().ok_or(ControllerError::MissingMux)?;

        match msg {
            MotorMessage::StartUpdates => {
                state.updates_handle =
                    Some(mux.send_interval(Duration::from_millis(1000), move || {
                        MuxMessage::Write(Packet::MotorAskStatus { slave })
                    }));
            }
            MotorMessage::GracefulStop => {
                mux.send_message(MuxMessage::Write(Packet::MotorStop { slave, mode: 0x01 }))?;
            }
            MotorMessage::EmergencyStop => {
                mux.send_message(MuxMessage::Write(Packet::MotorStop {
                    slave: self.slave,
                    mode: 0x00,
                }))?;
            }
            MotorMessage::Keep(movement) => {
                debug!("Keeping motor {} with {:?}", self.slave, movement);

                state.keep(self.slave, movement)?;
            }
            MotorMessage::Spin(movement) => {
                debug!("Spinning motor {} with {:?}", self.slave, movement);

                state.spin(self.slave, movement)?;
            }
            MotorMessage::SetOutputs(outputs) => {
                mux.send_message(MuxMessage::Write(Packet::MotorSetOutputs {
                    slave: self.slave,
                    outputs: if outputs { 0x01 } else { 0x00 },
                }))?;
            }
            MotorMessage::GetMaxSpeed(reply) => {
                reply.send(state.max_speed)?;
            }
            MotorMessage::Packet(packet) => match packet {
                Packet::MotorStatus {
                    slave,
                    running,
                    position,
                    remaining,
                    ..
                } if slave == self.slave => {
                    state.status = if running == 1 {
                        MotorStatus::Spinning {
                            position,
                            remaining,
                        }
                    } else {
                        MotorStatus::Idle
                    };

                    debug!("Motor {} status: {:?}", self.slave, state.status);
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