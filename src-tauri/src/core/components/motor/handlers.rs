use ractor::{async_trait, concurrency::Duration, Actor, ActorProcessingErr, ActorRef};
use std::sync::Arc;

use crate::core::{
    components::{Component, Handler, SpawnArgs},
    protocol::Packet,
    store::MotorsLimits,
};
use crate::utils::error::Error;

use super::{messages::MotorMessage, state::MotorState};

pub struct Motor {
    pub slave: u8,
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
            status: super::messages::MotorStatus::Idle,
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
                    super::messages::MotorMessage::Packet(Packet::MotorAskStatus { slave })
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
                state.controller.send_message(
                    crate::core::components::controller::ControllerMessage::Forward(
                        Packet::MotorStop {
                            slave,
                            gentle: true,
                        },
                    ),
                )?;
            }
            MotorMessage::EmergencyStop => {
                state.controller.send_message(
                    crate::core::components::controller::ControllerMessage::Forward(
                        Packet::MotorStop {
                            slave,
                            gentle: false,
                        },
                    ),
                )?;
            }
            MotorMessage::Keep(mut movement) => {
                state.keep(slave, &mut movement)?;
            }
            MotorMessage::Spin(mut movement) => {
                state.spin(slave, &mut movement)?;
            }
            MotorMessage::SetOutputs(outputs) => {
                state.controller.send_message(
                    crate::core::components::controller::ControllerMessage::Forward(
                        Packet::MotorSetOutputs { slave, outputs },
                    ),
                )?;
            }
            MotorMessage::GetMaxSpeed(reply) => {
                reply.send(state.max_speed)?;
            }
            MotorMessage::ReloadSettings => {
                state.config = MotorsLimits::try_from(state.store.clone())?;
            }
            MotorMessage::Packet(_) => {}
        }

        Ok(())
    }
}
