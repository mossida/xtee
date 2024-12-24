use std::sync::Arc;

use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef, SupervisionEvent};
use serde::{Deserialize, Serialize};
use specta::Type;
use tracing::{error, warn};

use crate::{
    core::components::{motor::Motor, SpawnArgs},
    core::protocol::Packet,
    core::store::Store,
    utils::error::Error,
};

use super::{
    actuator::Actuator,
    master::{Event, MasterMessage},
    mux::{Mux, MuxArguments, MuxMessage, MuxTarget},
    Component,
};

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Controller {
    pub id: String,
    pub group: ControllerGroup,
    pub serial_port: String,
    pub baud_rate: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "kebab-case")]
pub enum ControllerStatus {
    Connected,
    Disconnected,
    Failed { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, Type)]
#[serde(rename_all = "kebab-case")]
pub enum ControllerGroup {
    Default,
    Motors,
}

impl From<ControllerGroup> for String {
    fn from(val: ControllerGroup) -> Self {
        match val {
            ControllerGroup::Default => "default".to_owned(),
            ControllerGroup::Motors => "motors".to_owned(),
        }
    }
}

impl From<ControllerGroup> for Vec<ControllerChild> {
    fn from(val: ControllerGroup) -> Self {
        match val {
            ControllerGroup::Default => vec![ControllerChild::Actuator(Actuator)],
            ControllerGroup::Motors => vec![
                ControllerChild::Motor(Motor { slave: 1 }),
                ControllerChild::Motor(Motor { slave: 2 }),
            ],
        }
    }
}

pub enum ControllerChild {
    Motor(Motor),
    Actuator(Actuator),
}

impl ControllerChild {
    async fn spawn(
        self,
        controller: ActorRef<ControllerMessage>,
        store: Arc<Store>,
    ) -> Result<MuxTarget, ActorProcessingErr> {
        let handler = match self {
            ControllerChild::Motor(component) => {
                let handler = component.spawn(SpawnArgs { controller, store }).await?;

                Box::new(handler) as MuxTarget
            }
            ControllerChild::Actuator(component) => {
                let handler = component.spawn(SpawnArgs { controller, store }).await?;

                Box::new(handler) as MuxTarget
            }
        };

        Ok(handler)
    }
}

pub enum ControllerMessage {
    Connect,
    Forward(Packet),
}

pub struct ControllerState {
    mux: Option<ActorRef<MuxMessage>>,
    store: Arc<Store>,
    restart_count: u32,
}

impl ControllerState {
    pub async fn spawn_children(
        &self,
        actor: ActorRef<ControllerMessage>,
        controller: Controller,
    ) -> Result<ActorRef<MuxMessage>, ActorProcessingErr> {
        let children = Vec::<ControllerChild>::from(controller.group);

        let children = futures::future::try_join_all(
            children
                .into_iter()
                .map(|child| child.spawn(actor.clone(), self.store.clone())),
        )
        .await?;

        let name = format!("mux-{}", controller.id);

        let (mux, _) = Actor::spawn_linked(
            Some(name),
            Mux,
            MuxArguments {
                port: controller.serial_port,
                baud_rate: controller.baud_rate,
                targets: children,
            },
            actor.get_cell(),
        )
        .await?;

        Ok(mux)
    }
}

#[async_trait]
impl Actor for Controller {
    type Msg = ControllerMessage;
    type State = ControllerState;
    type Arguments = Arc<Store>;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        myself.send_message(ControllerMessage::Connect)?;

        Ok(ControllerState {
            restart_count: 0,
            store: args,
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
            ControllerMessage::Forward(message) => {
                state
                    .mux
                    .as_ref()
                    .ok_or(Error::MissingMux)?
                    .send_message(MuxMessage::Write(message))?;
            }
            ControllerMessage::Connect => {
                myself.stop_children(None);

                let master = myself.try_get_supervisor().ok_or(Error::Config)?;
                let mux = state.spawn_children(myself, self.clone()).await?;

                master.send_message(MasterMessage::Event(Event::ControllerStatus {
                    controller: self.clone(),
                    status: ControllerStatus::Connected,
                }))?;

                state.mux = Some(mux);
            }
        }

        Ok(())
    }

    async fn handle_supervisor_evt(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: SupervisionEvent,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            SupervisionEvent::ActorTerminated(who, _, _) => {
                warn!(
                    "Actor {} terminated",
                    who.get_name().unwrap_or(who.get_id().to_string())
                );

                if let Some(mux) = state.mux.as_ref() {
                    if mux.get_id() == who.get_id() {
                        if state.restart_count > 3 {
                            return Err(Error::MissingMux.into());
                        }

                        warn!("Mux terminated - attempting restart");
                        //myself.send_message(ControllerMessage::Spawn(ControllerChild::Mux))?;

                        state.restart_count += 1;
                    }
                }
            }
            SupervisionEvent::ActorFailed(who, err) => {
                error!(
                    "Actor {} failed because of {}",
                    who.get_name().unwrap_or(who.get_id().to_string()),
                    err
                );

                return Err(err);
            }
            _ => {}
        }

        Ok(())
    }
}
