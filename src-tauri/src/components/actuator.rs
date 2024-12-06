use std::{sync::Arc, time::Duration};

use pid::{ControlOutput, Pid};
use ractor::{async_trait, rpc, Actor, ActorProcessingErr, ActorRef, MessagingErr};
use tokio::task::JoinHandle;
use tracing::debug;

use crate::{
    components::master::Event,
    error::ControllerError,
    filter::KalmanFilter,
    protocol::Packet,
    store::{PIDSettings, Store, PID_SETTINGS, SCALE_GAIN},
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
                .ok_or(ControllerError::ConfigError)?
                .as_f64()
                .unwrap_or(0.0000672315) as f32,
            scale_offset: 0.0,
            output_limit: 200.0,
            pid_settings: settings,
        })
    }
}

pub enum ActuatorStatus {
    Loading,
    Keeping,
    Idle,
}

pub enum ActuatorMessage {
    Load(f32),
    Keep(f32),
    Move(u8),
    Stop,
    Packet(Packet),
}

impl From<Packet> for ActuatorMessage {
    fn from(value: Packet) -> Self {
        ActuatorMessage::Packet(value)
    }
}

pub struct ActuatorState {
    pid: Pid<f32>,
    mux: Option<Arc<ActorRef<MuxMessage>>>,
    //master: Arc<ActorRef<MasterMessage>>,
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
                self.current_step = self.step(value).ok();
            }
            ActuatorStatus::Loading => {
                if is_setpoint {
                    self.status = ActuatorStatus::Idle;
                } else {
                    self.current_step = self.step(value).ok();
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

            //self.master
            //    .send_message(MasterMessage::Event(Event::Weight(calculated)))?;

            match &self.current_step {
                Some(handle) => {
                    if handle.is_finished() {
                        self.handle_input(calculated);
                    }
                }
                None => self.handle_input(calculated),
            }
        }

        Ok(())
    }

    fn step(
        &mut self,
        value: f32,
    ) -> Result<JoinHandle<Result<(), MessagingErr<MuxMessage>>>, ActorProcessingErr> {
        let ControlOutput { output, .. } = self.pid.next_control_output(value);
        let pulse = (output.abs()) as u64;
        let mux = self.mux.clone().ok_or(ControllerError::MissingMux)?;

        debug!("Actuator step: {:.2}", output);

        mux.send_message(MuxMessage::Write(Packet::ActuatorMove {
            direction: if output > 0.0 { 0 } else { 1 },
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
        let filter = KalmanFilter::new(1.0, 1.0, 1.0, 1.0);

        //let master =
        //    registry::where_is("master".to_string()).ok_or(ControllerError::PacketError)?;

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
            filter,
            mux: None,
            //master: Arc::new(master.into()),
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
                .try_get_superivisor()
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
            ActuatorMessage::Load(value) => {
                state.status = ActuatorStatus::Loading;
                state.pid.setpoint(value);
            }
            ActuatorMessage::Keep(value) => {
                state.status = ActuatorStatus::Keeping;
                state.pid.setpoint(value);
            }
            ActuatorMessage::Move(direction) => {
                mux.send_message(MuxMessage::Write(Packet::ActuatorMove { direction }))?;
            }
            ActuatorMessage::Packet(packet) => state.handle_packet(packet)?,
        }

        Ok(())
    }
}
