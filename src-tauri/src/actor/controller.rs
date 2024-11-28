use std::sync::Arc;

use ractor::{
    async_trait, concurrency::JoinHandle, pg, Actor, ActorProcessingErr, ActorRef, SupervisionEvent,
};
use tauri::{AppHandle, Emitter};

use crate::{
    actor::motor::{Motor, MotorArguments},
    event::EVENT_COMPONENT_FAILED,
    store::{store, Store},
};

use super::{
    actuator::{Actuator, ActuatorArguments},
    motor::MotorMessage,
    mux::{Mux, MuxArguments, MuxStream, MuxTarget},
};

pub struct Controller;

pub enum ControllerChild {
    Mux,
    Actuator,
    Motor(u8),
}

impl ControllerChild {
    fn name(&self) -> String {
        match self {
            ControllerChild::Mux => "mux".to_string(),
            ControllerChild::Actuator => "actuator".to_string(),
            ControllerChild::Motor(slave) => format!("motor-{}", slave),
        }
    }

    async fn spawn(
        self,
        myself: ActorRef<ControllerMessage>,
        app: AppHandle,
        store: Arc<Store>,
    ) -> Result<(), ActorProcessingErr> {
        let name = self.name().to_string();

        match self {
            ControllerChild::Actuator => {
                Actuator::spawn_linked(
                    Some(name),
                    Actuator,
                    ActuatorArguments::try_from((store, app))?,
                    myself.get_cell(),
                )
                .await?;
            }
            ControllerChild::Mux => {
                let mut config = MuxArguments::try_from(store)?;

                config.targets = myself
                    .get_children()
                    .into_iter()
                    .map(|child| {
                        let target = Box::new(MuxTarget::from(child))
                            as Box<dyn ractor_actors::streams::Target<MuxStream>>;

                        target
                    })
                    .collect();

                Mux::spawn_linked(Some(name), Mux, config, myself.get_cell()).await?;
            }
            ControllerChild::Motor(slave) => {
                let (motor, _) = Motor::spawn_linked(
                    Some(format!("motor-{}", slave)),
                    Motor { slave },
                    MotorArguments {},
                    myself.get_cell(),
                )
                .await?;

                pg::join_scoped(
                    String::from("components"),
                    String::from("motors"),
                    vec![motor.get_cell()],
                );
            }
        };

        Ok(())
    }
}

pub enum ControllerMessage {
    Spawn(ControllerChild),
    Restart,
    PostSpawn,
}

pub struct ControllerState {
    app: AppHandle,
    store: Arc<Store>,
}

impl Controller {
    pub async fn spawn_children(
        controller: &ActorRef<ControllerMessage>,
    ) -> Result<(), ActorProcessingErr> {
        controller.send_message(ControllerMessage::Spawn(ControllerChild::Actuator))?;
        controller.send_message(ControllerMessage::Spawn(ControllerChild::Motor(1)))?;
        controller.send_message(ControllerMessage::Spawn(ControllerChild::Motor(2)))?;

        // Note: Mux must be spawned last because it needs the children to be spawned
        controller.send_message(ControllerMessage::Spawn(ControllerChild::Mux))?;
        controller.send_message(ControllerMessage::PostSpawn)?;

        Ok(())
    }

    pub async fn init(handle: AppHandle) -> Result<JoinHandle<()>, ActorProcessingErr> {
        let (_, handle) = Actor::spawn(Some("controller".to_owned()), Controller, handle).await?;

        Ok(handle)
    }
}

#[async_trait]
impl Actor for Controller {
    type Msg = ControllerMessage;
    type State = ControllerState;
    type Arguments = AppHandle;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let store = store(&args)?;

        myself.send_message(ControllerMessage::Restart)?;

        Ok(ControllerState { store, app: args })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            ControllerMessage::Spawn(child) => {
                child
                    .spawn(myself, state.app.clone(), state.store.clone())
                    .await?
            }
            ControllerMessage::Restart => {
                myself.stop_children(None);
                Controller::spawn_children(&myself).await?;
            }
            ControllerMessage::PostSpawn => {
                let motors =
                    pg::get_scoped_members(&String::from("components"), &String::from("motors"));

                for motor in motors {
                    let motor = ActorRef::<MotorMessage>::from(motor);

                    motor.send_message(MotorMessage::StartUpdates)?;
                }
            }
        }

        Ok(())
    }

    async fn handle_supervisor_evt(
        &self,
        _: ActorRef<Self::Msg>,
        message: SupervisionEvent,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            SupervisionEvent::ActorTerminated(who, _, _)
            | SupervisionEvent::ActorFailed(who, _) => {
                state
                    .app
                    .emit(EVENT_COMPONENT_FAILED, who.get_id().to_string())?;
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
