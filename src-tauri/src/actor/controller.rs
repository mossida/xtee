use std::sync::Arc;

use nanoid::nanoid;
use ractor::{
    async_trait, pg, Actor, ActorCell, ActorProcessingErr, ActorRef, RpcReplyPort, SupervisionEvent,
};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tracing::{error, warn};

use crate::{
    actor::motor::{Motor, MotorArguments},
    error::ControllerError,
    store::Store,
};

use super::{
    actuator::{Actuator, ActuatorArguments},
    master::SCOPE,
    mux::{Mux, MuxArguments, MuxMessage},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Controller {
    pub id: String,
    pub group: ControllerGroup,
    pub serial_port: String,
    pub baud_rate: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum ControllerGroup {
    Default,
    Motors,
}

impl Into<String> for ControllerGroup {
    fn into(self) -> String {
        match self {
            ControllerGroup::Default => "default".to_owned(),
            ControllerGroup::Motors => "motors".to_owned(),
        }
    }
}

impl Into<Vec<ControllerChild>> for ControllerGroup {
    fn into(self) -> Vec<ControllerChild> {
        match self {
            ControllerGroup::Default => vec![ControllerChild::Actuator],
            ControllerGroup::Motors => vec![ControllerChild::Motor(1), ControllerChild::Motor(2)],
        }
    }
}

pub enum ControllerChild {
    Mux,
    Actuator,
    Motor(u8),
}

impl ControllerChild {
    fn name(&self) -> String {
        match self {
            ControllerChild::Mux => nanoid!(4),
            ControllerChild::Actuator => "actuator".to_owned(),
            ControllerChild::Motor(slave) => format!("motor-{}", slave),
        }
    }

    async fn spawn(
        &self,
        myself: ActorRef<ControllerMessage>,
        controller: Controller,
        args: (Arc<Store>, AppHandle),
    ) -> Result<ActorCell, ActorProcessingErr> {
        let name = self.name();

        match self {
            ControllerChild::Actuator => {
                let (actuator, _) = Actuator::spawn_linked(
                    Some(name),
                    Actuator {
                        controller: myself.get_cell(),
                    },
                    ActuatorArguments::try_from(args)?,
                    myself.get_cell(),
                )
                .await?;

                pg::join_scoped(
                    SCOPE.to_owned(),
                    controller.group.into(),
                    vec![actuator.get_cell()],
                );

                Ok(actuator.get_cell())
            }
            ControllerChild::Mux => {
                let group = controller.group.clone();
                let config = MuxArguments::try_from(controller)?;

                let (mux, _) =
                    Mux::spawn_linked(Some(name), Mux, config, myself.get_cell()).await?;

                pg::join_scoped(SCOPE.to_owned(), group.into(), vec![mux.get_cell()]);

                Ok(mux.get_cell())
            }
            ControllerChild::Motor(slave) => {
                let (motor, _) = Motor::spawn_linked(
                    Some(name),
                    Motor { slave: *slave },
                    MotorArguments {},
                    myself.get_cell(),
                )
                .await?;

                pg::join_scoped(
                    SCOPE.to_owned(),
                    controller.group.into(),
                    vec![motor.get_cell()],
                );

                Ok(motor.get_cell())
            }
        }
    }
}

pub enum ControllerMessage {
    Start,
    Spawn(ControllerChild),
    FetchMux(RpcReplyPort<ActorRef<MuxMessage>>),
}

pub struct ControllerState {
    app: AppHandle,
    mux: Option<ActorRef<MuxMessage>>,
    store: Arc<Store>,
}

impl Controller {
    pub async fn spawn_children(
        &self,
        controller: &ActorRef<ControllerMessage>,
    ) -> Result<(), ActorProcessingErr> {
        let children: Vec<ControllerChild> = self.group.clone().into();

        for child in children {
            controller.send_message(ControllerMessage::Spawn(child))?;
        }

        controller.send_message(ControllerMessage::Spawn(ControllerChild::Mux))?;

        Ok(())
    }
}

#[async_trait]
impl Actor for Controller {
    type Msg = ControllerMessage;
    type State = ControllerState;
    type Arguments = (Arc<Store>, AppHandle);

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        myself.send_message(ControllerMessage::Start)?;

        Ok(ControllerState {
            store: args.0,
            app: args.1,
            mux: None,
        })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            ControllerMessage::FetchMux(reply) => {
                reply.send(state.mux.clone().ok_or(ControllerError::MissingMux)?);
            }
            ControllerMessage::Spawn(child) => {
                let cell = child
                    .spawn(
                        myself,
                        self.clone(),
                        (state.store.clone(), state.app.clone()),
                    )
                    .await
                    .inspect_err(|e| error!("Failed to spawn child: {}", e))?;

                match child {
                    ControllerChild::Mux => state.mux = Some(cell.into()),
                    _ => {}
                }
            }
            ControllerMessage::Start => {
                myself.stop_children(None);
                self.spawn_children(&myself).await?;
            }
        }

        Ok(())
    }

    async fn handle_supervisor_evt(
        &self,
        _: ActorRef<Self::Msg>,
        message: SupervisionEvent,
        _state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            SupervisionEvent::ActorTerminated(who, _, _) => {
                warn!(
                    "Actor {} terminated",
                    who.get_name().unwrap_or(who.get_id().to_string())
                );
            }
            SupervisionEvent::ActorFailed(who, err) => {
                error!(
                    "Actor {} failed because of {}",
                    who.get_name().unwrap_or(who.get_id().to_string()),
                    err
                );
            }
            _ => {}
        }

        Ok(())
    }

    async fn post_stop(
        &self,
        _: ActorRef<Self::Msg>,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        state.store.close_resource();

        Ok(())
    }
}
