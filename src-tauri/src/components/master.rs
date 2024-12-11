use std::{collections::HashMap, sync::Arc};

use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef, RpcReplyPort, SupervisionEvent};
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Emitter};
use tracing::{error, warn};

use crate::{
    error::ControllerError,
    store::{store, Store, CONTROLLERS},
};

use super::{
    controller::{Controller, ControllerGroup, ControllerMessage},
    motor::MotorStatus,
};

pub const SCOPE: &str = "components";

pub struct Master;

#[derive(Clone, Type, Serialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "kebab-case")]
pub enum Event {
    Weight(f64),
    MotorStatus(MotorStatus),
}

pub struct MasterState {
    pub app: AppHandle,
    pub store: Arc<Store>,
    pub refs: HashMap<String, ActorRef<ControllerMessage>>,
    pub ports: HashMap<String, bool>,
    pub groups: HashMap<ControllerGroup, bool>,
    pub controllers: HashMap<String, Controller>,
}

pub enum MasterMessage {
    Restart,
    Spawn(Controller),
    #[allow(dead_code)]
    Event(Event),
    FetchControllers(RpcReplyPort<Vec<Controller>>),
}

#[async_trait]
impl Actor for Master {
    type Msg = MasterMessage;
    type State = MasterState;
    type Arguments = AppHandle;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let store = store(&args)?;
        let controllers = store
            .get(CONTROLLERS)
            .unwrap_or(serde_json::Value::Array(vec![]));

        let controllers: Vec<Controller> = serde_json::from_value(controllers)?;

        for controller in controllers {
            myself.send_message(MasterMessage::Spawn(controller))?;
        }

        Ok(MasterState {
            app: args,
            refs: HashMap::new(),
            store,
            ports: HashMap::new(),
            groups: HashMap::new(),
            controllers: HashMap::new(),
        })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            MasterMessage::Spawn(controller) => {
                if state.groups.contains_key(&controller.group)
                    || state.ports.contains_key(&controller.serial_port)
                {
                    return Err(ControllerError::ConfigError.into());
                }

                state.groups.insert(controller.group.clone(), true);
                state.ports.insert(controller.serial_port.clone(), true);

                let id = controller.id.clone();
                let result = Actor::spawn_linked(
                    Some(id.clone()),
                    controller.clone(),
                    (state.store.clone(), state.app.clone()),
                    myself.get_cell(),
                )
                .await;

                match result {
                    Ok((actor_ref, _)) => {
                        state.refs.insert(id.clone(), actor_ref);
                        state.controllers.insert(id, controller);
                    }
                    Err(e) => error!("Failed to spawn controller: {}", e),
                }
            }
            MasterMessage::FetchControllers(reply) => {
                reply.send(state.controllers.values().cloned().collect())?;
            }
            MasterMessage::Event(event) => {
                state.app.emit("app:event", event)?;
            }
            _ => {}
        };

        Ok(())
    }

    async fn handle_supervisor_evt(
        &self,
        myself: ActorRef<Self::Msg>,
        message: SupervisionEvent,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            SupervisionEvent::ActorTerminated(who, _, _) => {
                warn!(
                    "Actor {} terminated",
                    who.get_name().unwrap_or(who.get_id().to_string())
                );

                if let Some(id) = who.get_name() {
                    let controller = state.controllers.remove(&id);

                    if let Some(controller) = controller {
                        state.groups.remove(&controller.group);
                        state.ports.remove(&controller.serial_port);
                    }

                    state.refs.remove(&id);
                }
            }
            SupervisionEvent::ActorFailed(who, err) => {
                error!(
                    "Actor {} failed because of {}",
                    who.get_name().unwrap_or(who.get_id().to_string()),
                    err
                );

                if let Some(id) = who.get_name() {
                    let controller = state.controllers.remove(&id);

                    if let Some(controller) = controller {
                        state.groups.remove(&controller.group);
                        state.ports.remove(&controller.serial_port);
                    }

                    state.refs.remove(&id);
                }
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
