use std::{sync::Arc, time::Duration};

use pid_lite::Controller as Pid;
use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef, MessagingErr};

use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::task::JoinHandle;
use tracing::{debug, info};

use crate::{
    core::components::master::Event,
    core::protocol::Packet,
    core::store::{PIDSettings, Store, StoreKey},
    utils::error::Error,
};

use super::{controller::ControllerMessage, master::MasterMessage, Component, Handler, SpawnArgs};

pub struct Actuator;

pub struct ActuatorConfig {
    pub precision: f64,
    pub scale_gain: f64,
    pub scale_offset: f64,
    pub pid_settings: PIDSettings,
    pub max_load: f64,
    pub min_load: f64,
}

impl TryFrom<Arc<Store>> for ActuatorConfig {
    type Error = Error;

    fn try_from(store: Arc<Store>) -> Result<Self, Self::Error> {
        let pid_settings = store
            .get(StoreKey::ActuatorPidSettings)
            .ok_or(Error::Config)?;

        let settings: PIDSettings = serde_json::from_value(pid_settings)?;

        let scale_gain = store.get(StoreKey::ScaleGain).ok_or(Error::Config)?;
        let scale_offset = store.get(StoreKey::ScaleOffset).ok_or(Error::Config)?;

        let max_load = store.get(StoreKey::ActuatorMaxLoad).ok_or(Error::Config)?;
        let min_load = store.get(StoreKey::ActuatorMinLoad).ok_or(Error::Config)?;
        let precision = store
            .get(StoreKey::ActuatorPrecision)
            .ok_or(Error::Config)?;

        Ok(Self {
            precision: precision.as_f64().ok_or(Error::InvalidStore)?,
            scale_gain: scale_gain.as_f64().ok_or(Error::InvalidStore)?,
            max_load: max_load.as_f64().ok_or(Error::InvalidStore)?,
            min_load: min_load.as_f64().ok_or(Error::InvalidStore)?,
            scale_offset: scale_offset.as_f64().ok_or(Error::InvalidStore)?,
            pid_settings: settings,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Type)]
#[serde(tag = "status", content = "data")]
#[serde(rename_all = "kebab-case")]
pub enum ActuatorStatus {
    Loading { target: f32 },
    Keeping { target: f32 },
    Idle,
}

pub enum ActuatorMessage {
    Load(f32),
    Keep(f32),
    Move(bool),
    Stop,
    Packet(Packet),
    ReloadSettings,
}

impl From<Packet> for ActuatorMessage {
    fn from(value: Packet) -> Self {
        ActuatorMessage::Packet(value)
    }
}

pub struct ActuatorState {
    pid: Pid,
    store: Arc<Store>,
    master: ActorRef<MasterMessage>,
    controller: ActorRef<ControllerMessage>,
    status: ActuatorStatus,
    config: ActuatorConfig,
    current_step: Option<JoinHandle<Result<(), MessagingErr<ControllerMessage>>>>,
    current_offset: Option<f64>,
}

impl ActuatorState {
    fn send_status(&self) -> Result<(), ActorProcessingErr> {
        self.master
            .send_message(MasterMessage::Event(Event::ActuatorStatus(self.status)))?;

        Ok(())
    }

    fn apply_config(&mut self) {
        let pid_settings = &self.config.pid_settings;

        self.pid
            .set_proportional_gain(pid_settings.proportional as f64);
        self.pid.set_integral_gain(pid_settings.integral as f64);
        self.pid.set_derivative_gain(pid_settings.derivative as f64);

        self.current_offset = Some(self.config.scale_offset);

        info!("Applied new config");
    }

    fn handle_input(&mut self, value: f64) -> Result<(), ActorProcessingErr> {
        let target = self.pid.target();
        let is_setpoint = (value - target).abs() < self.config.precision;

        if value > self.config.max_load {
            return self.stop();
        }

        match self.status {
            ActuatorStatus::Keeping { .. } if !is_setpoint => {
                self.current_step = self.step_pid(value).ok();
            }
            ActuatorStatus::Loading { .. } => {
                if is_setpoint {
                    self.stop()?;
                } else {
                    self.current_step = self.step_pid(value).ok();
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_packet(&mut self, packet: Packet) -> Result<(), ActorProcessingErr> {
        if let Packet::Data { value } = packet {
            let raw = (value as f64) * self.config.scale_gain;
            let offset = self.current_offset.get_or_insert(raw);
            let value = raw - *offset;

            self.master
                .send_message(MasterMessage::Event(Event::Weight(value)))?;

            return match &self.current_step {
                Some(handle) => {
                    handle.abort();
                    self.handle_input(value)
                }
                None => self.handle_input(value),
            };
        }

        Ok(())
    }

    fn step_pid(
        &mut self,
        value: f64,
    ) -> Result<JoinHandle<Result<(), MessagingErr<ControllerMessage>>>, ActorProcessingErr> {
        let correction = self.pid.update(value);

        debug!(
            "Actuator step pid with value: {:.2} kg, correction: {:.2} ms",
            value, correction
        );

        self.step(correction)
    }

    fn step(
        &self,
        value_ms: f64,
    ) -> Result<JoinHandle<Result<(), MessagingErr<ControllerMessage>>>, ActorProcessingErr> {
        let pulse = ((value_ms.abs()).clamp(8.0, 200.0) * 1000.0) as u64;

        debug!("Moving for: {} microseconds", pulse);

        self.controller
            .send_message(ControllerMessage::Forward(Packet::ActuatorMove {
                direction: value_ms < 0.0,
            }))?;

        Ok(self
            .controller
            .send_after(Duration::from_micros(pulse), || {
                ControllerMessage::Forward(Packet::ActuatorStop)
            }))
    }

    fn stop(&mut self) -> Result<(), ActorProcessingErr> {
        self.controller
            .send_message(ControllerMessage::Forward(Packet::ActuatorStop))?;

        self.status = ActuatorStatus::Idle;
        self.send_status()?;

        Ok(())
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
        match message {
            ActuatorMessage::Stop => {
                state.status = ActuatorStatus::Idle;

                state.current_step.take().inspect(|handle| {
                    handle.abort();
                });

                state
                    .controller
                    .send_message(ControllerMessage::Forward(Packet::ActuatorStop))?;

                state.send_status()?;
            }
            ActuatorMessage::Load(value) => {
                state.status = ActuatorStatus::Loading { target: value };

                let settings = &state.config.pid_settings;
                let target = (value as f64).clamp(state.config.min_load, state.config.max_load);

                state.pid.reset();
                state.pid.set_target(target);

                info!("Loading: {:.2} kg with settings: {:?}", value, settings);

                state.send_status()?;
            }
            ActuatorMessage::Keep(value) => {
                state.status = ActuatorStatus::Keeping { target: value };

                let target = (value as f64).clamp(state.config.min_load, state.config.max_load);

                state.pid.reset();
                state.pid.set_target(target);

                state.send_status()?;
            }
            ActuatorMessage::Move(direction) => {
                state.controller.send_message(ControllerMessage::Forward(
                    Packet::ActuatorMove { direction },
                ))?;
            }
            ActuatorMessage::Packet(packet) => state.handle_packet(packet)?,
            ActuatorMessage::ReloadSettings => {
                debug!("Reloading settings");

                state.config = ActuatorConfig::try_from(state.store.clone())?;
                state.apply_config();
            }
        }

        Ok(())
    }
}
