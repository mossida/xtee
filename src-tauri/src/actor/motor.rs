use ractor::{
    async_trait,
    concurrency::{Duration, JoinHandle},
    rpc, Actor, ActorCell, ActorProcessingErr, ActorRef,
};
use tracing::debug;

use crate::{actor::controller::ControllerMessage, error::ControllerError, protocol::Packet};

use super::mux::MuxMessage;

pub struct Motor {
    pub slave: u8,
    pub controller: ActorCell,
}

#[derive(Debug, Clone)]
pub struct MotorMovement {
    pub direction: u8,
    pub rotations: u16,
    pub speed: u16,
}

#[derive(Debug, Clone)]
pub enum MotorMessage {
    StartUpdates,
    Spin(MotorMovement),
    GracefulStop,
    EmergencyStop,
    Packet(Packet),
}

impl From<Packet> for MotorMessage {
    fn from(packet: Packet) -> Self {
        MotorMessage::Packet(packet)
    }
}

#[derive(Debug, Clone)]
pub enum MotorStatus {
    Idle,
    Spinning { position: u32, remaining: u32 },
}

#[derive(Debug)]
pub struct MotorState {
    status: MotorStatus,
    max_speed: u32,
    updates_handle: Option<JoinHandle<()>>,
    mux: Option<ActorRef<MuxMessage>>,
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
        _myself: ActorRef<Self::Msg>,
        msg: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        if state.mux.is_none() {
            let mux = rpc::call(&self.controller, ControllerMessage::FetchMux, None)
                .await?
                .success_or(ControllerError::MissingMux)?;

            debug!("Motor got mux: {:?}", mux.get_name());

            state.mux = Some(mux);
        }

        match msg {
            MotorMessage::StartUpdates => {
                let slave = self.slave;
                state.updates_handle = Some(
                    state
                        .mux
                        .as_ref()
                        .ok_or(ControllerError::MissingMux)?
                        .send_interval(Duration::from_millis(1000), move || {
                            MuxMessage::Write(Packet::MotorAskStatus { slave })
                        }),
                );
            }
            MotorMessage::GracefulStop => {
                state
                    .mux
                    .as_ref()
                    .ok_or(ControllerError::MissingMux)?
                    .send_message(MuxMessage::Write(Packet::MotorStop {
                        slave: self.slave,
                        mode: 0x01,
                    }))?;
            }
            MotorMessage::EmergencyStop => {
                state
                    .mux
                    .as_ref()
                    .ok_or(ControllerError::MissingMux)?
                    .send_message(MuxMessage::Write(Packet::MotorStop {
                        slave: self.slave,
                        mode: 0x00,
                    }))?;
            }
            MotorMessage::Spin(movement) => {
                debug!("Spinning motor {} with {:?}", self.slave, movement);

                /*mux.send_message(MuxMessage::Write(Packet::MotorSettings {
                    slave: self.slave,
                    speed: movement.speed,
                    acceleration: 1000,
                }))?;*/

                state
                    .mux
                    .as_ref()
                    .ok_or(ControllerError::MissingMux)?
                    .send_message(MuxMessage::Write(Packet::MotorMove {
                        slave: self.slave,
                        direction: movement.direction,
                        rotations: movement.rotations,
                    }))?;
            }
            MotorMessage::Packet(packet) => match packet {
                Packet::MotorStatus {
                    slave,
                    running,
                    max_speed,
                    position,
                    remaining,
                    ..
                } if slave == self.slave => {
                    state.max_speed = max_speed;

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
