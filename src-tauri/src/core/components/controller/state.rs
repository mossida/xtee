use std::sync::Arc;

use ractor::{Actor, ActorProcessingErr, ActorRef};

use crate::core::store::Store;

use crate::core::components::mux::{Mux, MuxArguments, MuxMessage};

use super::handlers::Controller;
use super::messages::{ControllerChild, ControllerMessage};

pub struct ControllerState {
    pub mux: Option<ActorRef<MuxMessage>>,
    pub store: Arc<Store>,
    pub restart_count: u32,
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
