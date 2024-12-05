use std::{collections::HashMap, sync::Arc};

use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef, RpcReplyPort};
use serde::Serialize;
use specta::Type;
use tauri::AppHandle;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

use crate::{
    error::ControllerError,
    store::{store, Store, CONTROLLERS},
};

use super::controller::{Controller, ControllerGroup, ControllerMessage};

pub const SCOPE: &str = "components";

pub struct Master;

#[derive(Clone, Type, Serialize)]
#[serde(tag = "type", content = "data")]
#[specta(rename_all = "kebab-case")]
pub enum Event {
    Weight(f32),
}

pub struct MasterState {
    pub app: AppHandle,
    pub store: Arc<Store>,
    pub refs: HashMap<String, ActorRef<ControllerMessage>>,
    pub ports: HashMap<String, bool>,
    pub groups: HashMap<ControllerGroup, bool>,
    pub controllers: HashMap<String, Controller>,
    pub channel: broadcast::Sender<Event>,
}

pub enum MasterMessage {
    Restart,
    Spawn(Controller),
    Event(Event),
    FetchControllers(RpcReplyPort<Vec<Controller>>),
    FetchStream(RpcReplyPort<BroadcastStream<Event>>),
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
            channel: broadcast::channel(16).0,
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
                let (actor_ref, _) = Actor::spawn_linked(
                    Some(id.clone()),
                    controller.clone(),
                    (state.store.clone(), state.app.clone()),
                    myself.get_cell(),
                )
                .await?;

                state.refs.insert(id.clone(), actor_ref);
                state.controllers.insert(id, controller);
            }
            MasterMessage::FetchControllers(reply) => {
                reply.send(state.controllers.values().cloned().collect())?;
            }
            MasterMessage::FetchStream(reply) => {
                reply.send(BroadcastStream::new(state.channel.subscribe()))?;
            }
            MasterMessage::Event(event) => {
                state
                    .channel
                    .send(event)
                    .map_err(|_| ControllerError::PacketError)?;
            }
            _ => {}
        };

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
