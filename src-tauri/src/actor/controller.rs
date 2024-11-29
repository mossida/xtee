use std::sync::Arc;

use ractor::{
    async_trait, concurrency::JoinHandle, pg, Actor, ActorProcessingErr, ActorRef, SupervisionEvent,
};
use tauri::{AppHandle, Emitter};
use tracing::{error, warn};

use crate::{
    actor::motor::{Motor, MotorArguments},
    event::EVENT_COMPONENT_FAILED,
    store::{store, Store},
};

use super::{
    actuator::{Actuator, ActuatorArguments},
    mux::{Mux, MuxArguments},
};

pub const COMPONENTS_SCOPE: &str = "components";

pub const MOTORS_GROUP: &str = "motors";
pub const DEFAULT_GROUP: &str = "default";

// TODO: add motors group
pub const ALL_GROUPS: &[&str] = &[DEFAULT_GROUP];

pub struct Controller;

pub enum ControllerChild {
    Mux { group: String },
    Actuator,
    Motor(u8),
}

impl ControllerChild {
    fn name(&self) -> String {
        match self {
            ControllerChild::Mux { group } => format!("mux-{}", group),
            ControllerChild::Actuator => "actuator".to_owned(),
            ControllerChild::Motor(slave) => format!("motor-{}", slave),
        }
    }

    async fn spawn(
        self,
        myself: ActorRef<ControllerMessage>,
        app: AppHandle,
        store: Arc<Store>,
    ) -> Result<(), ActorProcessingErr> {
        let name = self.name();

        match self {
            ControllerChild::Actuator => {
                Actuator::spawn_linked(
                    Some(name),
                    Actuator {
                        group: DEFAULT_GROUP.to_owned(),
                    },
                    ActuatorArguments::try_from((store, app))?,
                    myself.get_cell(),
                )
                .await?;
            }
            ControllerChild::Mux { group } => {
                let config = MuxArguments::try_from((store, group.clone()))?;

                let (mux, _) =
                    Mux::spawn_linked(Some(name), Mux, config, myself.get_cell()).await?;

                pg::join_scoped(COMPONENTS_SCOPE.to_owned(), group, vec![mux.get_cell()]);
            }
            ControllerChild::Motor(slave) => {
                let (motor, _) = Motor::spawn_linked(
                    Some(format!("motor-{}", slave)),
                    Motor { slave },
                    MotorArguments {},
                    myself.get_cell(),
                )
                .await?;

                // TODO: move to self join
                pg::join_scoped(
                    COMPONENTS_SCOPE.to_owned(),
                    MOTORS_GROUP.to_owned(),
                    vec![motor.get_cell()],
                );
            }
        };

        Ok(())
    }
}

pub enum ControllerMessage {
    Start,
    Spawn(ControllerChild),
}

pub struct ControllerState {
    app: AppHandle,
    store: Arc<Store>,
}

impl Controller {
    pub async fn spawn_children(
        controller: &ActorRef<ControllerMessage>,
    ) -> Result<(), ActorProcessingErr> {
        controller.send_message(ControllerMessage::Spawn(ControllerChild::Actuator))?;
        controller.send_message(ControllerMessage::Spawn(ControllerChild::Motor(1)))?;
        controller.send_message(ControllerMessage::Spawn(ControllerChild::Motor(2)))?;

        // Spawn muxes for each group
        for group in ALL_GROUPS {
            controller.send_message(ControllerMessage::Spawn(ControllerChild::Mux {
                group: group.to_string(),
            }))?;
        }

        Ok(())
    }

    pub async fn init(handle: AppHandle) -> Result<JoinHandle<()>, ActorProcessingErr> {
        let (_, handle) = Actor::spawn(Some("controller".to_owned()), Controller, handle).await?;

        Ok(handle)
    }
}

#[async_trait]
impl Actor for Controller {
    type Msg = ControllerMessage;
    type State = ControllerState;
    type Arguments = AppHandle;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let store = store(&args)?;

        myself.send_message(ControllerMessage::Start)?;

        Ok(ControllerState { store, app: args })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            ControllerMessage::Spawn(child) => {
                child
                    .spawn(myself, state.app.clone(), state.store.clone())
                    .await
                    .inspect_err(|e| error!("Failed to spawn child: {}", e))?;
            }
            ControllerMessage::Start => {
                myself.stop_children(None);
                Controller::spawn_children(&myself).await?;
            }
        }

        Ok(())
    }

    async fn handle_supervisor_evt(
        &self,
        _: ActorRef<Self::Msg>,
        message: SupervisionEvent,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            SupervisionEvent::ActorTerminated(who, _, _) => {
                warn!("Actor {} terminated", who.get_id());
            }
            SupervisionEvent::ActorFailed(who, err) => {
                error!("Actor {} failed because of {}", who.get_id(), err);

                state
                    .app
                    .emit(EVENT_COMPONENT_FAILED, who.get_id().to_string())?;
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
