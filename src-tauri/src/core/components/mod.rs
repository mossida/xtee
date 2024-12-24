use std::sync::Arc;

use controller::ControllerMessage;
use mux::MuxStream;
use ractor::{Actor, ActorProcessingErr, ActorRef, Message};
use ractor_actors::streams::Target;

use crate::{core::protocol::Packet, core::store::Store};

pub mod actuator;
pub mod controller;
pub mod master;
pub mod motor;
pub mod mux;

pub struct Handler<T: Message + From<Packet>> {
    pub cell: ActorRef<T>,
}

impl<T: Message + From<Packet>> Target<MuxStream> for Handler<T> {
    fn get_id(&self) -> String {
        self.cell.get_id().to_string()
    }

    fn message_received(
        &self,
        item: <MuxStream as futures::Stream>::Item,
    ) -> Result<(), ActorProcessingErr> {
        self.cell.send_message(T::from(item))?;

        Ok(())
    }
}

pub struct SpawnArgs {
    store: Arc<Store>,
    controller: ActorRef<ControllerMessage>,
}

trait Component: Actor<Msg: From<Packet>> {
    async fn spawn(self, args: SpawnArgs) -> Result<Handler<Self::Msg>, ActorProcessingErr>;
}
