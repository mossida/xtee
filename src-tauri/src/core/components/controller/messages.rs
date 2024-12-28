use std::sync::Arc;

use ractor::{ActorProcessingErr, ActorRef};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::core::{
    components::{
        actuator::Actuator, motor::Motor, mux::MuxTarget, Component, SpawnArgs, Stoppable,
    },
    protocol::Packet,
    store::Store,
};

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

pub enum ControllerMessage {
    Connect,
    Forward(Packet),
}

pub enum ControllerChild {
    Motor(Motor),
    Actuator(Actuator),
}

impl ControllerChild {
    pub fn stoppable(&self) -> Packet {
        match self {
            ControllerChild::Motor(motor) => motor.packet(),
            ControllerChild::Actuator(actuator) => actuator.packet(),
        }
    }
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

impl ControllerChild {
    pub async fn spawn(
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
