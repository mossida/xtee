use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use pid_lite::Controller as Pid;
use ractor::{async_trait, registry, rpc, Actor, ActorProcessingErr, ActorRef, MessagingErr};
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

use crate::{
    components::master::Event,
    error::ControllerError,
    protocol::Packet,
    store::{PIDSettings, Store, StoreKey},
    tuner::Tuner,
};

use super::{controller::ControllerMessage, master::MasterMessage, mux::MuxMessage};

pub struct Actuator;

pub struct ActuatorArguments {
    pub precision: f64,
    pub scale_gain: f64,
    pub scale_offset: f64,
    pub pid_settings: PIDSettings,
    store: Arc<Store>,
}

impl ActuatorArguments {
    pub fn reload(&mut self) -> Result<(), ControllerError> {
        *self = Self::try_from(self.store.clone())?;

        Ok(())
    }
}

impl TryFrom<Arc<Store>> for ActuatorArguments {
    type Error = ControllerError;

    fn try_from(value: Arc<Store>) -> Result<Self, Self::Error> {
        let pid_settings = value
            .get(StoreKey::ActuatorPidSettings)
            .ok_or(ControllerError::ConfigError)?;

        let settings: PIDSettings = serde_json::from_value(pid_settings)?;

        Ok(ActuatorArguments {
            precision: 1.5,
            scale_gain: value
                .get(StoreKey::ScaleGain)
                .ok_or(ControllerError::InvalidStore)?
                .as_f64()
                .ok_or(ControllerError::InvalidStore)?,
            scale_offset: 0.0,
            pid_settings: settings,
            store: value,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActuatorStatus {
    Loading,
    Keeping,
    Tuning,
    Idle,
}

pub enum ActuatorMessage {
    Load(f32),
    Keep(f32),
    Move(u8),
    Stop,
    Packet(Packet),
    Tune,
    ReloadSettings,
}

impl From<Packet> for ActuatorMessage {
    fn from(value: Packet) -> Self {
        ActuatorMessage::Packet(value)
    }
}

pub struct ActuatorState {
    pid: Pid,
    tuner: Tuner,
    mux: Option<Arc<ActorRef<MuxMessage>>>,
    master: Arc<ActorRef<MasterMessage>>,
    status: ActuatorStatus,
    config: ActuatorArguments,
    current_step: Option<JoinHandle<Result<(), MessagingErr<MuxMessage>>>>,
    current_offset: Option<f64>,
    last_update: Option<Instant>,
}

impl ActuatorState {
    fn handle_input(&mut self, value: f64) {
        let target = self.pid.target();
        let is_setpoint = (value - target).abs() < self.config.precision;

        match self.status {
            ActuatorStatus::Keeping if !is_setpoint => {
                self.current_step = self.step_pid(value).ok();
            }
            ActuatorStatus::Loading => {
                if is_setpoint {
                    self.status = ActuatorStatus::Idle;
                } else {
                    self.current_step = self.step_pid(value).ok();
                }
            }
            ActuatorStatus::Tuning => {
                if self.tuner.is_preload_ok() || self.tuner.verify_preload(value) {
                    if self.tuner.is_tuning_complete() {
                        self.status = ActuatorStatus::Idle;

                        let parameters = self.tuner.get_pid_parameters().unwrap();

                        info!("Tuning complete: {:?}", parameters);

                        self.pid.reset();
                        self.pid.set_proportional_gain(parameters.kp);
                        self.pid.set_integral_gain(parameters.ki);
                        self.pid.set_derivative_gain(parameters.kd);

                        self.tuner.reset();
                    } else {
                        let output = self.tuner.process_measurement(value);
                        self.current_step = self.step(output).ok();
                    }
                } else {
                    warn!("Please preload the scale");
                }
            }
            _ => {}
        }
    }

    fn handle_packet(&mut self, packet: Packet) -> Result<(), ActorProcessingErr> {
        if let Packet::Data { value } = packet {
            let raw = (value as f64) * self.config.scale_gain;
            let offset = self.current_offset.get_or_insert(raw);
            let value = raw - *offset;

            let now = Instant::now();
            let elapsed = self
                .last_update
                .map(|last| now.duration_since(last))
                .unwrap_or(Duration::from_millis(1));

            self.last_update = Some(now);

            debug!("Elapsed between data packet: {:?}", elapsed);

            self.master
                .send_message(MasterMessage::Event(Event::Weight(value)))?;

            match &self.current_step {
                Some(handle) => {
                    handle.abort();

                    self.handle_input(value);
                }
                None => self.handle_input(value),
                _ => {}
            }
        }

        Ok(())
    }

    fn step_pid(
        &mut self,
        value: f64,
    ) -> Result<JoinHandle<Result<(), MessagingErr<MuxMessage>>>, ActorProcessingErr> {
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
    ) -> Result<JoinHandle<Result<(), MessagingErr<MuxMessage>>>, ActorProcessingErr> {
        let mux = self.mux.clone().ok_or(ControllerError::MissingMux)?;
        let pulse = ((value_ms.abs()).clamp(10.0, 200.0) * 1000.0) as u64;

        debug!("Moving for: {} microseconds", pulse);

        mux.send_message(MuxMessage::Write(Packet::ActuatorMove {
            direction: if value_ms > 0.0 { 0 } else { 1 },
        }))?;

        Ok(mux.send_after(Duration::from_micros(pulse), || {
            MuxMessage::Write(Packet::ActuatorStop)
        }))
    }
}

#[async_trait]
impl Actor for Actuator {
    type Msg = ActuatorMessage;
    type State = ActuatorState;
    type Arguments = ActuatorArguments;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        config: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let tuner = Tuner::new(88.0, 100.0);
        let mut pid = Pid::new(0.0, 0.0, 0.0, 0.0);

        let master =
            registry::where_is("master".to_string()).ok_or(ControllerError::PacketError)?;

        let PIDSettings {
            proportional,
            integral,
            derivative,
            ..
        } = config.pid_settings;

        pid.set_proportional_gain(proportional as f64);
        pid.set_integral_gain(integral as f64);
        pid.set_derivative_gain(derivative as f64);

        Ok(ActuatorState {
            pid,
            tuner,
            mux: None,
            master: Arc::new(master.into()),
            status: ActuatorStatus::Idle,
            current_step: None,
            current_offset: Some(config.scale_offset),
            config,
            last_update: None,
        })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        if state.mux.is_none() {
            let controller = myself
                .try_get_supervisor()
                .ok_or(ControllerError::ConfigError)?;

            let mux = rpc::call(&controller, ControllerMessage::FetchMux, None)
                .await?
                .success_or(ControllerError::MissingMux)?;

            debug!("Actuator got mux: {:?}", mux.get_name());

            state.mux = Some(Arc::new(mux));
        }

        let mux = state.mux.as_ref().ok_or(ControllerError::MissingMux)?;

        match message {
            ActuatorMessage::Stop => {
                state.status = ActuatorStatus::Idle;

                state.current_step.take().inspect(|handle| {
                    handle.abort();
                });

                mux.send_message(MuxMessage::Write(Packet::ActuatorStop))?;
            }
            ActuatorMessage::Tune => {
                state.status = ActuatorStatus::Tuning;
            }
            ActuatorMessage::Load(value) if state.status != ActuatorStatus::Tuning => {
                state.status = ActuatorStatus::Loading;
                state.pid.reset();
                state.pid.set_target(value as f64);
            }
            ActuatorMessage::Keep(value) if state.status != ActuatorStatus::Tuning => {
                state.status = ActuatorStatus::Keeping;
                state.pid.reset();
                state.pid.set_target(value as f64);
            }
            ActuatorMessage::Move(direction) if state.status != ActuatorStatus::Tuning => {
                mux.send_message(MuxMessage::Write(Packet::ActuatorMove { direction }))?;
            }
            ActuatorMessage::Packet(packet) => state.handle_packet(packet)?,
            ActuatorMessage::ReloadSettings => {
                state.config.reload()?;
                state.current_offset = Some(state.config.scale_offset);
            }
            _ => {}
        }

        Ok(())
    }
}
