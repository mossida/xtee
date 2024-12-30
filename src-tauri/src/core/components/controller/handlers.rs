use std::sync::Arc;

use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef, SupervisionEvent};
use serde::{Deserialize, Serialize};
use specta::Type;
use tracing::{debug, error, info, instrument, warn};

use crate::{core::store::Store, utils::error::Error};

use crate::core::components::{
    master::{Event, MasterMessage},
    mux::MuxMessage,
};

use super::messages::{ControllerGroup, ControllerMessage, ControllerStatus};
use super::state::ControllerState;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Controller {
    pub id: String,
    pub group: ControllerGroup,
    pub serial_port: String,
    pub baud_rate: u32,
}

#[async_trait]
impl Actor for Controller {
    type Msg = ControllerMessage;
    type State = ControllerState;
    type Arguments = Arc<Store>;

    #[instrument(skip(self, myself, args), fields(controller_id = %myself.get_id()))]
    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        debug!("Starting controller");
        myself.send_message(ControllerMessage::Connect)?;

        Ok(ControllerState {
            restart_count: 0,
            store: args,
            mux: None,
        })
    }

    #[instrument(skip(self, myself, state), fields(controller_id = %myself.get_id()))]
    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        debug!(?message, "Handling message");

        match message {
            ControllerMessage::Forward(message) => {
                debug!("Forwarding message to mux");
                state
                    .mux
                    .as_ref()
                    .ok_or(Error::MissingMux)?
                    .send_message(MuxMessage::Write(message))?;
            }
            ControllerMessage::Connect => {
                info!("Connecting controller");
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

    #[instrument(skip(self, myself, state), fields(controller_id = %myself.get_id()))]
    async fn handle_supervisor_evt(
        &self,
        myself: ActorRef<Self::Msg>,
        message: SupervisionEvent,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            SupervisionEvent::ActorTerminated(who, _, _) => {
                warn!(
                    actor = %who.get_name().unwrap_or(who.get_id().to_string()),
                    "Actor terminated"
                );

                if let Some(mux) = state.mux.as_ref() {
                    if mux.get_id() == who.get_id() {
                        if state.restart_count > 3 {
                            return Err(Error::MissingMux.into());
                        }

                        debug!(
                            restart_count = state.restart_count,
                            "Attempting to reconnect mux"
                        );
                        myself.send_message(ControllerMessage::Connect)?;
                        state.restart_count += 1;
                    }
                }
            }
            SupervisionEvent::ActorFailed(who, err) => {
                error!(
                    actor = %who.get_name().unwrap_or(who.get_id().to_string()),
                    error = %err,
                    "Actor failed"
                );

                return Err(err);
            }
            _ => {}
        }

        Ok(())
    }
}
