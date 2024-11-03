use std::sync::Arc;

use pid::{ControlOutput, Pid};
use ractor::{async_trait, Actor, ActorProcessingErr, ActorRef};
use serde_json::Value;

use crate::{config, store::Store};

use super::operator::{Direction, Move, Operator};

pub struct Actuator;

pub struct ActuatorConfig {
    pub pid: (f32, f32, f32),
    pub precision: f32,
    pub output_limit: f32,
}

impl From<Arc<Store>> for ActuatorConfig {
    fn from(store: Arc<Store>) -> Self {
        let pid_proportional = store.get(config::PID_PROPORTIONAL).unwrap();
        let pid_integral = store.get(config::PID_INTEGRAL).unwrap();
        let pid_derivative = store.get(config::PID_DERIVATIVE).unwrap();

        ActuatorConfig {
            pid: (
                pid_proportional.as_f64().unwrap_or(1.0) as f32,
                pid_integral.as_f64().unwrap_or(0.0) as f32,
                pid_derivative.as_f64().unwrap_or(0.0) as f32,
            ),
            precision: 0.01,
            output_limit: 100.0,
        }
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
    Move(Move),
    GracefulStop,
    EmergencyStop,
}

pub struct ActuatorState {
    pid: Pid<f32>,
    status: ActuatorStatus,
    operator: ActorRef<<Operator as ractor::Actor>::Msg>,
    config: ActuatorConfig,
}

impl ActuatorState {
    fn step(&mut self, value: f32) -> Result<(), ActorProcessingErr> {
        let ControlOutput { output, .. } = self.pid.next_control_output(value);

        let direction = if output > 0.0 {
            Direction::Forward
        } else {
            Direction::Backward
        };

        self.operator.send_message(Move {
            direction,
            duration: output,
        })?;

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
        myself: ActorRef<Self::Msg>,
        config: Self::Arguments,
    ) -> Result<Self::State, ActorProcessingErr> {
        let (operator, _) = Actor::spawn_linked(None, Operator, (), myself.get_cell()).await?;
        let mut pid = Pid::new(0.0, config.output_limit);

        // TODO: Understand if limit is the same as the gain
        pid.p(config.pid.0, config.pid.0);
        pid.i(config.pid.1, config.pid.1);
        pid.d(config.pid.2, config.pid.2);

        Ok(ActuatorState {
            pid,
            status: ActuatorStatus::Idle,
            operator,
            config,
        })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> Result<(), ActorProcessingErr> {
        match message {
            ActuatorMessage::Move(move_msg) => {
                state.operator.send_message(move_msg)?;
            }
            ActuatorMessage::EmergencyStop => {
                state.status = ActuatorStatus::Idle;
                state.operator.kill();
                state.operator = Actor::spawn_linked(None, Operator, (), myself.get_cell())
                    .await?
                    .0;
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
