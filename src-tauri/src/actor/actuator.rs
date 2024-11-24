use std::sync::Arc;

use pid::{ControlOutput, Pid};
use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef};

use crate::{
    error::ControllerError,
    store::{Store, PID_DERIVATIVE, PID_INTEGRAL, PID_PROPORTIONAL},
};

pub struct Actuator;

pub struct ActuatorConfig {
    pub pid: (f32, f32, f32),
    pub precision: f32,
    pub output_limit: f32,
}

impl TryFrom<Arc<Store>> for ActuatorConfig {
    type Error = ControllerError;

    fn try_from(value: Arc<Store>) -> Result<Self, Self::Error> {
        let pid_p_value = value
            .get(PID_PROPORTIONAL)
            .ok_or(ControllerError::ConfigError)?;
        let pid_i_value = value
            .get(PID_INTEGRAL)
            .ok_or(ControllerError::ConfigError)?;
        let pid_d_value = value
            .get(PID_DERIVATIVE)
            .ok_or(ControllerError::ConfigError)?;

        let pid_p = pid_p_value.as_f64().unwrap() as f32;
        let pid_i = pid_i_value.as_f64().unwrap() as f32;
        let pid_d = pid_d_value.as_f64().unwrap() as f32;

        Ok(ActuatorConfig {
            pid: (pid_p, pid_i, pid_d),
            precision: 0.0,
            output_limit: 0.0,
        })
    }
}

pub enum ActuatorStatus {
    Loading,
    Keeping,
    Idle,
}

pub enum ActuatorMessage {
    Data(f32),
    Load(f32),
    Keep(f32),
    GracefulStop,
    EmergencyStop,
}

pub struct ActuatorState {
    pid: Pid<f32>,
    status: ActuatorStatus,
    config: ActuatorConfig,
}

impl ActuatorState {
    fn step(&mut self, value: f32) -> Result<(), ActorProcessingErr> {
        let ControlOutput { output, .. } = self.pid.next_control_output(value);

        Ok(())
    }
}

#[async_trait]
impl Actor for Actuator {
    type Msg = ActuatorMessage;
    type State = ActuatorState;
    type Arguments = ActuatorConfig;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        config: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let mut pid = Pid::new(0.0, config.output_limit);

        // TODO: Understand if limit is the same as the gain
        pid.p(config.pid.0, config.pid.0);
        pid.i(config.pid.1, config.pid.1);
        pid.d(config.pid.2, config.pid.2);

        Ok(ActuatorState {
            pid,
            status: ActuatorStatus::Idle,

            config,
        })
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            ActuatorMessage::EmergencyStop => {
                state.status = ActuatorStatus::Idle;
            }
            ActuatorMessage::GracefulStop => {
                state.status = ActuatorStatus::Idle;
            }
            ActuatorMessage::Load(value) => {
                state.status = ActuatorStatus::Loading;
                state.pid.setpoint(value);
            }
            ActuatorMessage::Keep(value) => {
                state.status = ActuatorStatus::Keeping;
                state.pid.setpoint(value);
            }
            ActuatorMessage::Data(value) => match state.status {
                ActuatorStatus::Keeping => state.step(value)?,
                ActuatorStatus::Loading => {
                    if (value - state.pid.setpoint).abs() < 0.01 {
                        state.status = ActuatorStatus::Idle;
                    } else {
                        state.step(value)?;
                    }
                }
                _ => {}
            },
        }

        Ok(())
    }
}
