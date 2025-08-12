use ractor::{Actor, ActorProcessingErr, ActorRef, concurrency::Duration};
use tracing::debug;

use crate::{
    core::{
        components::{
            Component, Handler, SpawnArgs, Stoppable,
            controller::ControllerMessage,
            master::{Event, MasterMessage},
            motor::state::MotorStatus,
        },
        protocol::Packet,
    },
    utils::error::Error,
};

use super::{MotorMessage, MotorsLimits, state::MotorState};

pub struct Motor {
    pub slave: u8,
}

impl Stoppable for Motor {
    fn packet(&self) -> Packet {
        Packet::MotorStop {
            slave: self.slave,
            gentle: false,
        }
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
                    outputs,
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

                    debug!(
                        "Motor {} status: {:?}, outputs: {}",
                        self.slave, state.status, outputs
                    );

                    state
                        .master
                        .send_message(MasterMessage::Event(Event::MotorStatus(
                            self.slave,
                            state.status.clone(),
                            outputs,
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
