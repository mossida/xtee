use std::{sync::Arc, time::Duration};

use pid::{ControlOutput, Pid};
use ractor::{async_trait, pg, registry, Actor, ActorProcessingErr, ActorRef};
use tauri::{AppHandle, Emitter};
use tokio::task::JoinHandle;
use tracing::{debug, trace};

use crate::{
    error::ControllerError,
    event::EVENT_WEIGHT,
    filter::KalmanFilter,
    protocol::Packet,
    store::{PIDSettings, Store, PID_SETTINGS},
};

use super::{controller::COMPONENTS_SCOPE, mux::MuxMessage};

pub struct Actuator {
    pub group: String,
}

pub struct ActuatorArguments {
    pub precision: f32,
    pub output_limit: f32,
    pub pid_settings: PIDSettings,
    pub handle: AppHandle,
}

impl TryFrom<(Arc<Store>, AppHandle)> for ActuatorArguments {
    type Error = ControllerError;

    fn try_from((value, handle): (Arc<Store>, AppHandle)) -> Result<Self, Self::Error> {
        let pid_settings = value
            .get(PID_SETTINGS)
            .ok_or(ControllerError::ConfigError)?;

        let settings: PIDSettings = serde_json::from_value(pid_settings)?;

        Ok(ActuatorArguments {
            precision: 0.25,
            output_limit: 200.0,
            pid_settings: settings,
            handle,
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
    mux: Option<ActorRef<MuxMessage>>,
    filter: KalmanFilter,
    status: ActuatorStatus,
    config: ActuatorArguments,
    current_step: Option<JoinHandle<Result<(), ActorProcessingErr>>>,
    current_offset: Option<f32>,
}

impl ActuatorState {
    fn handle_input(&mut self, value: f32) {
        match self.status {
            ActuatorStatus::Keeping => {
                if (value - self.pid.setpoint).abs() < self.config.precision {
                    // Do nothing
                } else {
                    self.current_step = Some(self.step(value));
                }
            }
            ActuatorStatus::Loading => {
                if (value - self.pid.setpoint).abs() < self.config.precision {
                    self.status = ActuatorStatus::Idle;
                } else {
                    self.current_step = Some(self.step(value));
                }
            }
            _ => {}
        }
    }

    fn handle_packet(&mut self, packet: Packet) -> Result<(), ActorProcessingErr> {
        if let Packet::Data { value } = packet {
            let raw = (value as f32) * 0.0000672315;
            //let offset = self.current_offset.get_or_insert(raw);
            let calculated = self.filter.update(raw);

            self.config.handle.emit(EVENT_WEIGHT, calculated)?;

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

    fn step(&mut self, value: f32) -> JoinHandle<Result<(), ActorProcessingErr>> {
        let ControlOutput { output, .. } = self.pid.next_control_output(value);
        let pulse = (output.abs().clamp(5.0, 50.0)) as u64;
        let mux = self.mux.clone().ok_or(ControllerError::MissingMux);

        debug!("Actuator step: {:.2}", output);

        tokio::spawn(async move {
            let mux = mux?;

            mux.send_message(MuxMessage::Write(Packet::ActuatorMove {
                direction: if output > 0.0 { 0 } else { 1 },
            }))?;

            tokio::time::sleep(Duration::from_millis(pulse)).await;

            mux.send_message(MuxMessage::Write(Packet::ActuatorStop))?;

            trace!("Actuator step finished");

            Ok::<(), ActorProcessingErr>(())
        })
    }
}

#[async_trait]
impl Actor for Actuator {
    type Msg = ActuatorMessage;
    type State = ActuatorState;
    type Arguments = ActuatorArguments;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        config: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let mut pid = Pid::new(0.0, config.output_limit);
        let filter = KalmanFilter::new(1.0, 1.0, 1.0, 1.0);

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

        pg::join_scoped(
            COMPONENTS_SCOPE.to_owned(),
            self.group.clone(),
            vec![myself.get_cell()],
        );

        Ok(ActuatorState {
            pid,
            filter,
            mux: None,
            status: ActuatorStatus::Idle,
            current_step: None,
            current_offset: None,
            config,
        })
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        if state.mux.is_none() {
            let mux = registry::where_is(format!("mux-{}", self.group))
                .ok_or(ControllerError::MissingMux)?;
            state.mux = Some(mux.into());
        }

        match message {
            ActuatorMessage::Stop => {
                state.status = ActuatorStatus::Idle;

                state.current_step.take().inspect(|handle| {
                    handle.abort();
                });

                state
                    .mux
                    .as_ref()
                    .ok_or(ControllerError::MissingMux)?
                    .send_message(MuxMessage::Write(Packet::ActuatorStop))?;
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
                state
                    .mux
                    .as_ref()
                    .ok_or(ControllerError::MissingMux)?
                    .send_message(MuxMessage::Write(Packet::ActuatorMove { direction }))?;
            }
            ActuatorMessage::Packet(packet) => state.handle_packet(packet)?,
        }

        Ok(())
    }
}
