use pid_lite::Controller as Pid;
use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef};

use tracing::{debug, info};

use crate::{
    core::{
        components::{controller::ControllerMessage, Component, Handler, SpawnArgs, Stoppable},
        protocol::Packet,
    },
    utils::error::Error,
};

use super::{
    config::ActuatorConfig,
    messages::{ActuatorMessage, ActuatorStatus},
    state::ActuatorState,
    ActuatorDirection,
};

pub struct Actuator;

impl Stoppable for Actuator {
    fn packet(&self) -> Packet {
        Packet::ActuatorStop
    }
}

impl Component for Actuator {
    async fn spawn(self, args: SpawnArgs) -> Result<Handler<ActuatorMessage>, ActorProcessingErr> {
        let cell = args.controller.get_cell();
        let (actuator, _) =
            Actuator::spawn_linked(Some("actuator".to_owned()), self, args, cell).await?;

        Ok(Handler { cell: actuator })
    }
}

#[async_trait]
impl Actor for Actuator {
    type Msg = ActuatorMessage;
    type State = ActuatorState;
    type Arguments = SpawnArgs;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        SpawnArgs { store, controller }: SpawnArgs,
    ) -> Result<Self::State, ActorProcessingErr> {
        let pid = Pid::new(0.0, 0.0, 0.0, 0.0);
        let config = ActuatorConfig::try_from(store.clone())?;
        let master = controller
            .try_get_supervisor()
            .ok_or(Error::MissingAncestor)?;

        Ok(ActuatorState {
            pid,
            store,
            master: master.into(),
            controller,
            status: ActuatorStatus::Idle,
            current_step: None,
            current_offset: Some(config.scale_offset),
            config,
            bypass: false,
        })
    }

    async fn post_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        state.apply_config();
        state.send_status()?;

        Ok(())
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        let overloaded = state.status == ActuatorStatus::Overloaded;

        match message {
            ActuatorMessage::Stop => {
                if !overloaded {
                    state.status = ActuatorStatus::Idle;
                }

                state.bypass = false;
                state.current_step.take().inspect(|handle| {
                    handle.abort();
                });

                state
                    .controller
                    .send_message(ControllerMessage::Forward(Packet::ActuatorStop))?;

                state.send_status()?;
            }
            ActuatorMessage::Load(value) if !overloaded => {
                state.status = ActuatorStatus::Loading { target: value };

                let settings = &state.config.pid_settings;
                let target = (value as f64).clamp(state.config.min_load, state.config.max_load);

                state.pid.reset();
                state.pid.set_target(target);

                info!("Loading: {:.2} kg with settings: {:?}", value, settings);

                state.send_status()?;
            }
            ActuatorMessage::Keep(value) if !overloaded => {
                state.status = ActuatorStatus::Keeping { target: value };

                let target = (value as f64).clamp(state.config.min_load, state.config.max_load);

                state.pid.reset();
                state.pid.set_target(target);

                state.send_status()?;
            }
            ActuatorMessage::Unload => {
                state.status = ActuatorStatus::Unloading;
                state.controller.send_message(ControllerMessage::Forward(
                    ActuatorDirection::unload().into_packet(),
                ))?;

                state.send_status()?;
            }
            ActuatorMessage::Move(movement) if !overloaded || movement.is_unload() => {
                state.bypass = overloaded && movement.is_unload();
                state
                    .controller
                    .send_message(ControllerMessage::Forward(movement.into_packet()))?;
            }
            ActuatorMessage::Packet(packet) => state.handle_packet(packet)?,
            ActuatorMessage::ReloadSettings => {
                debug!("Reloading settings");

                state.config = ActuatorConfig::try_from(state.store.clone())?;
                state.apply_config();
            }
            _ => {}
        }

        Ok(())
    }
}
