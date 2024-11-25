use std::sync::Arc;

use ractor::{
    async_trait, concurrency::JoinHandle, registry, Actor, ActorProcessingErr, ActorRef,
    SupervisionEvent,
};
use tauri::{AppHandle, Emitter};

use crate::{
    protocol::Packet,
    store::{store, Store},
};

use super::{
    actuator::{Actuator, ActuatorConfig},
    mux::{Mux, MuxArguments, MuxStream, MuxTarget},
};

pub struct Controller;

pub enum ControllerChild {
    Mux,
    Actuator,
}

impl ControllerChild {
    fn name(&self) -> &str {
        match self {
            ControllerChild::Mux => "mux",
            ControllerChild::Actuator => "actuator",
        }
    }

    async fn spawn(
        self,
        myself: ActorRef<ControllerMessage>,
        store: Arc<Store>,
    ) -> Result<(), ActorProcessingErr> {
        let name = self.name().to_string();

        match self {
            ControllerChild::Actuator => {
                Actuator::spawn_linked(
                    Some(name),
                    Actuator,
                    ActuatorConfig::try_from(store)?,
                    myself.get_cell(),
                )
                .await?;
            }
            ControllerChild::Mux => {
                let mut config = MuxArguments::try_from(store)?;

                config.targets = myself
                    .get_children()
                    .into_iter()
                    .map(|child| {
                        let target = Box::new(MuxTarget::from(child))
                            as Box<dyn ractor_actors::streams::Target<MuxStream>>;

                        target
                    })
                    .collect();

                Mux::spawn_linked(Some(name), Mux, config, myself.get_cell()).await?;
            }
        };

        Ok(())
    }
}

pub enum ControllerMessage {
    Spawn(ControllerChild),
}

pub struct ControllerState {
    app: AppHandle,
    store: Arc<Store>,
}

impl Controller {
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

        myself.send_message(ControllerMessage::Spawn(ControllerChild::Actuator))?;
        myself.send_message(ControllerMessage::Spawn(ControllerChild::Mux))?;

        Ok(ControllerState { store, app: args })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            ControllerMessage::Spawn(child) => child.spawn(myself, state.store.clone()).await?,
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
            SupervisionEvent::ActorTerminated(who, _, _)
            | SupervisionEvent::ActorFailed(who, _) => {
                state
                    .app
                    .emit("controller-error", who.get_id().to_string())?;
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
