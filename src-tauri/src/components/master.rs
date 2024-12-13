use std::{collections::HashMap, sync::Arc};

use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef, RpcReplyPort, SupervisionEvent};
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Emitter};
use tracing::{error, warn};

use crate::store::{store, Store, StoreKey};

use super::{
    actuator::ActuatorStatus,
    controller::{Controller, ControllerStatus},
    motor::MotorStatus,
};

pub const SCOPE: &str = "components";

pub struct Master;

#[derive(Clone, Type, Serialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "kebab-case")]
pub enum Event {
    Init,
    Weight(f64),
    MotorStatus(MotorStatus),
    ActuatorStatus(ActuatorStatus),
    ControllerStatus {
        controller: Controller,
        status: ControllerStatus,
    },
}

pub struct MasterState {
    pub app: AppHandle,
    pub store: Arc<Store>,
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
            .get(StoreKey::Controllers)
            .unwrap_or(serde_json::Value::Array(vec![]));

        let controllers: Vec<Controller> = serde_json::from_value(controllers)?;

        if controllers.is_empty() {
            warn!("No controllers found");
        }

        for controller in controllers {
            myself.send_message(MasterMessage::Spawn(controller))?;
        }

        Ok(MasterState {
            app: args,
            store,
            controllers: HashMap::new(),
        })
    }

    async fn post_start(
        &self,
        myself: ActorRef<Self::Msg>,
        _state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        myself.send_message(MasterMessage::Event(Event::Init))?;

        Ok(())
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            MasterMessage::Spawn(controller) => {
                let id = controller.id.clone();
                let result = Actor::spawn_linked(
                    Some(id.clone()),
                    controller.clone(),
                    (state.store.clone(), state.app.clone()),
                    myself.get_cell(),
                )
                .await;

                match result {
                    Ok((_, _)) => {
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
                let id = who.get_name().unwrap_or(who.get_id().to_string());
                let controller = state.controllers.remove(&id);

                who.get_children().iter().for_each(|child| {
                    child.kill();
                });

                if let Some(controller) = controller {
                    myself.send_message(MasterMessage::Event(Event::ControllerStatus {
                        controller,
                        status: ControllerStatus::Disconnected,
                    }))?;
                }
            }
            SupervisionEvent::ActorFailed(who, error) => {
                let id = who.get_name().unwrap_or(who.get_id().to_string());
                let controller = state.controllers.remove(&id);

                who.get_children().iter().for_each(|child| {
                    child.kill();
                });

                if let Some(controller) = controller {
                    myself.send_message(MasterMessage::Event(Event::ControllerStatus {
                        controller,
                        status: ControllerStatus::Failed {
                            reason: error.to_string(),
                        },
                    }))?;
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
