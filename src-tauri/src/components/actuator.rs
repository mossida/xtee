use std::{sync::Arc, time::Duration};

use pid::{ControlOutput, Pid};
use ractor::{
    async_trait, registry, rpc, Actor, ActorProcessingErr, ActorRef, MessagingErr, RpcReplyPort,
};
use tokio::task::JoinHandle;
use tracing::{debug, warn};

use crate::{
    components::master::Event,
    error::ControllerError,
    filter::KalmanFilter,
    protocol::Packet,
    store::{PIDSettings, Store, PID_SETTINGS, SCALE_GAIN},
    tuner::Tuner,
};

use super::{controller::ControllerMessage, master::MasterMessage, mux::MuxMessage};

pub struct Actuator;

pub struct ActuatorArguments {
    pub precision: f32,
    pub scale_gain: f32,
    pub scale_offset: f32,
    pub output_limit: f32,
    pub pid_settings: PIDSettings,
}

impl TryFrom<Arc<Store>> for ActuatorArguments {
    type Error = ControllerError;

    fn try_from(value: Arc<Store>) -> Result<Self, Self::Error> {
        let pid_settings = value
            .get(PID_SETTINGS)
            .ok_or(ControllerError::ConfigError)?;

        let settings: PIDSettings = serde_json::from_value(pid_settings)?;

        Ok(ActuatorArguments {
            precision: 0.25,
            scale_gain: value
                .get(SCALE_GAIN)
                .ok_or(ControllerError::InvalidStore)?
                .as_f64()
                .ok_or(ControllerError::InvalidStore)? as f32,
            scale_offset: 0.0,
            output_limit: 200.0,
            pid_settings: settings,
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
}

impl From<Packet> for ActuatorMessage {
    fn from(value: Packet) -> Self {
        ActuatorMessage::Packet(value)
    }
}

pub struct ActuatorState {
    pid: Pid<f32>,
    tuner: Tuner,
    mux: Option<Arc<ActorRef<MuxMessage>>>,
    master: Arc<ActorRef<MasterMessage>>,
    filter: KalmanFilter,
    status: ActuatorStatus,
    config: ActuatorArguments,
    current_step: Option<JoinHandle<Result<(), MessagingErr<MuxMessage>>>>,
    current_offset: Option<f32>,
}

impl ActuatorState {
    fn handle_input(&mut self, value: f32) {
        let is_setpoint = (value - self.pid.setpoint).abs() < self.config.precision;

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
                if self.tuner.verify_preload(value) {
                    if self.tuner.is_tuning_complete() {
                        self.status = ActuatorStatus::Idle;

                        let parameters = self.tuner.get_pid_parameters().unwrap();

                        self.pid.p(parameters.kp, parameters.kp);
                        self.pid.i(parameters.ki, parameters.ki);
                        self.pid.d(parameters.kd, parameters.kd);

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
            let raw = (value as f32) * self.config.scale_gain;
            let offset = self.current_offset.get_or_insert(raw);
            let calculated = self.filter.update(raw - *offset);

            debug!("Actuator handle packet: {:.2}", calculated);

            self.master
                .send_message(MasterMessage::Event(Event::Weight(calculated)))?;

            match &self.current_step {
                Some(handle) if handle.is_finished() => {
                    self.handle_input(calculated);
                }
                None => self.handle_input(calculated),
                _ => {}
            }
        }

        Ok(())
    }

    fn step_pid(
        &mut self,
        value: f32,
    ) -> Result<JoinHandle<Result<(), MessagingErr<MuxMessage>>>, ActorProcessingErr> {
        let ControlOutput { output, .. } = self.pid.next_control_output(value);

        debug!("Actuator step: {:.2}", output);

        self.step(output)
    }

    fn step(
        &self,
        value: f32,
    ) -> Result<JoinHandle<Result<(), MessagingErr<MuxMessage>>>, ActorProcessingErr> {
        let mux = self.mux.clone().ok_or(ControllerError::MissingMux)?;
        let pulse = (value.abs()) as u64;

        debug!("Moving by: {:?}", pulse);

        mux.send_message(MuxMessage::Write(Packet::ActuatorMove {
            direction: if value > 0.0 { 0 } else { 1 },
        }))?;

        Ok(mux.send_after(Duration::from_millis(pulse), || {
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
        let mut pid = Pid::new(0.0, config.output_limit);
        let tuner = Tuner::new(200.0, 50.0);
        let filter = KalmanFilter::new(1.0, 1.0, 1.0, 1.0);

        let master =
            registry::where_is("master".to_string()).ok_or(ControllerError::PacketError)?;

        let PIDSettings {
            proportional,
            integral,
            derivative,
            derivative_limit,
            proportional_limit,
            integral_limit,
        } = config.pid_settings;

        // TODO: Understand if limit is the same as the gain
        pid.p(proportional, proportional_limit);
        pid.i(integral, integral_limit);
        pid.d(derivative, derivative_limit);

        Ok(ActuatorState {
            pid,
            tuner,
            filter,
            mux: None,
            master: Arc::new(master.into()),
            status: ActuatorStatus::Idle,
            current_step: None,
            current_offset: Some(config.scale_offset),
            config,
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
                state.pid.setpoint(value);
            }
            ActuatorMessage::Keep(value) if state.status != ActuatorStatus::Tuning => {
                state.status = ActuatorStatus::Keeping;
                state.pid.setpoint(value);
            }
            ActuatorMessage::Move(direction) if state.status != ActuatorStatus::Tuning => {
                mux.send_message(MuxMessage::Write(Packet::ActuatorMove { direction }))?;
            }
            ActuatorMessage::Packet(packet) => state.handle_packet(packet)?,
            _ => {}
        }

        Ok(())
    }
}
