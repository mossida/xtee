use std::{collections::HashMap, sync::Arc};

use ractor::{Actor, ActorProcessingErr, ActorRef, RpcReplyPort, SupervisionEvent, registry};
use serde::Serialize;
use specta::Type;
use tauri::{AppHandle, Emitter};
use tracing::{debug, error, warn};

use crate::core::store::{Store, StoreKey, store};

use super::{
    actuator::ActuatorStatus,
    controller::{Controller, ControllerChild, ControllerMessage, ControllerStatus},
    motor::MotorStatus,
};

pub struct Master;

#[derive(Clone, Type, Serialize)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "kebab-case")]
pub enum Event {
    Init,
    Weight(f64),
    MotorStatus(u8, MotorStatus),
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
    Kill(String),
    Spawn(Controller),
    Event(Event),
    FetchControllers(RpcReplyPort<Vec<Controller>>),
    SystemStop,
}

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

        debug!("Master initialized");

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
                let mut spawn_controller = async |ctrl: Controller| -> Result<(), String> {
                    match Actor::spawn_linked(
                        Some(ctrl.id.clone()),
                        ctrl.clone(),
                        state.store.clone(),
                        myself.get_cell(),
                    )
                    .await
                    {
                        Ok((_, _)) => {
                            state.controllers.insert(ctrl.id.clone(), ctrl);
                            Ok(())
                        }
                        Err(e) => Err(format!("Failed to spawn controller: {}", e)),
                    }
                };

                if let Err(e) = spawn_controller(controller).await {
                    error!("{}", e);
                }
            }
            MasterMessage::FetchControllers(reply) => {
                reply.send(state.controllers.values().cloned().collect())?;
            }
            MasterMessage::Event(event) => {
                state.app.emit("app:event", event)?;
            }
            MasterMessage::Kill(id) => {
                let controller = state.controllers.remove(&id);

                if let Some(controller) = controller {
                    let cell =
                        registry::where_is(controller.id.clone()).ok_or(rspc::Error::new(
                            rspc::ErrorCode::NotFound,
                            "Controller not found".to_owned(),
                        ))?;

                    cell.stop(None);

                    myself.send_message(MasterMessage::Event(Event::ControllerStatus {
                        controller,
                        status: ControllerStatus::Disconnected,
                    }))?;
                }
            }
            MasterMessage::SystemStop => {
                for controller in state.controllers.values() {
                    if let Some(actor) = registry::where_is(controller.id.clone()) {
                        for child in Vec::<ControllerChild>::from(controller.group.clone()) {
                            if let Err(e) =
                                actor.send_message(ControllerMessage::Forward(child.stoppable()))
                            {
                                error!("Failed to send stop packet to child: {}", e);
                            }
                        }
                    } else {
                        error!("Controller not found: {}", controller.id);
                    }
                }
            }
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
