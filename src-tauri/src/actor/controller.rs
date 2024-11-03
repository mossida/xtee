use std::sync::Arc;

use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef, SupervisionEvent};
use tauri::{AppHandle, Emitter};

use crate::{
    config,
    store::{store, Store},
};

use super::actuator::{Actuator, ActuatorConfig};

pub struct Controller;

pub enum ControllerChild {
    Actuator,
}

impl ControllerChild {
    fn name(&self) -> &str {
        match self {
            ControllerChild::Actuator => "actuator",
        }
    }

    async fn spawn(
        &self,
        myself: ActorRef<ControllerMessage>,
        store: Arc<Store>,
    ) -> Result<(), ActorProcessingErr> {
        match self {
            ControllerChild::Actuator => {
                Actuator::spawn_linked(
                    Some(self.name().to_string()),
                    Actuator,
                    ActuatorConfig::from(store),
                    myself.get_cell(),
                )
                .await?
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
    pub async fn spawn(handle: AppHandle) -> Result<(), ActorProcessingErr> {
        Actor::spawn(None, Controller, handle)
            .await?
            .1
            .await
            .map_err(|e| e.into())
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

        if store.has(config::FORWARD_PIN) && store.has(config::BACKWARD_PIN) {
            myself.send_message(ControllerMessage::Spawn(ControllerChild::Actuator))?;
        }

        Ok(ControllerState { store, app: args })
    }

    async fn handle(
        &self,
        _: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            ControllerMessage::Spawn(child) => match child {
                ControllerChild::Actuator => {}
            },
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
