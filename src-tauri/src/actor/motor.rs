use ractor::{
    async_trait,
    concurrency::{Duration, JoinHandle},
    registry, Actor, ActorProcessingErr, ActorRef,
};
use tracing::debug;

use crate::{error::ControllerError, protocol::Packet};

use super::mux::MuxMessage;

pub struct Motor {
    pub slave: u8,
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
        })
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        msg: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        let mux = ActorRef::<MuxMessage>::from(
            registry::where_is("mux".to_string()).ok_or(ControllerError::MissingMux)?,
        );

        match msg {
            MotorMessage::StartUpdates => {
                let slave = self.slave.clone();
                /*state.updates_handle =
                Some(mux.send_interval(Duration::from_millis(1000), move || {
                    MuxMessage::Write(Packet::MotorAskStatus { slave })
                }));*/
            }
            MotorMessage::GracefulStop => {
                mux.send_message(MuxMessage::Write(Packet::MotorStop {
                    slave: self.slave,
                    mode: 0x01,
                }))?;
            }
            MotorMessage::EmergencyStop => {
                mux.send_message(MuxMessage::Write(Packet::MotorStop {
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

                mux.send_message(MuxMessage::Write(Packet::MotorMove {
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
