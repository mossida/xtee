use std::{sync::Arc, time::Duration};

use pid_lite::Controller as Pid;
use ractor::{ActorProcessingErr, ActorRef, MessagingErr};

use tokio::task::JoinHandle;
use tracing::{debug, info};

use crate::core::{
    components::{
        actuator::ActuatorDirection,
        controller::ControllerMessage,
        master::{Event, MasterMessage},
    },
    protocol::Packet,
    store::Store,
};

use super::{config::ActuatorConfig, messages::ActuatorStatus};

pub struct ActuatorState {
    pub pid: Pid,
    pub store: Arc<Store>,
    pub master: ActorRef<MasterMessage>,
    pub controller: ActorRef<ControllerMessage>,
    pub status: ActuatorStatus,
    pub config: ActuatorConfig,
    pub current_step: Option<JoinHandle<Result<(), MessagingErr<ControllerMessage>>>>,
    pub current_offset: Option<f64>,
    pub bypass: bool,
}

impl ActuatorState {
    pub fn send_status(&self) -> Result<(), ActorProcessingErr> {
        self.master
            .send_message(MasterMessage::Event(Event::ActuatorStatus(self.status)))?;

        Ok(())
    }

    pub fn apply_config(&mut self) {
        let pid_settings = &self.config.pid_settings;

        self.pid
            .set_proportional_gain(pid_settings.proportional as f64);
        self.pid.set_integral_gain(pid_settings.integral as f64);
        self.pid.set_derivative_gain(pid_settings.derivative as f64);

        self.current_offset = Some(self.config.scale_offset);

        info!("Applied new config");
    }

    pub fn handle_input(&mut self, value: f64) -> Result<(), ActorProcessingErr> {
        let target = self.pid.target();
        let is_setpoint = (value - target).abs() < self.config.precision;

        if value > self.config.max_load && !self.bypass {
            self.status = ActuatorStatus::Overloaded;
            return self.system_stop();
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
            ActuatorStatus::Overloaded if !self.bypass => {
                self.status = ActuatorStatus::Idle;
                self.send_status()?;
            }
            ActuatorStatus::Unloading => {
                if value < self.config.precision {
                    self.stop()?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    pub fn handle_packet(&mut self, packet: Packet) -> Result<(), ActorProcessingErr> {
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
        let direction = if value_ms < 0.0 {
            ActuatorDirection::unload()
        } else {
            ActuatorDirection::load()
        };

        debug!("Moving for: {} microseconds", pulse);

        self.controller
            .send_message(ControllerMessage::Forward(direction.into_packet()))?;

        Ok(self
            .controller
            .send_after(Duration::from_micros(pulse), || {
                ControllerMessage::Forward(Packet::ActuatorStop)
            }))
    }

    fn system_stop(&mut self) -> Result<(), ActorProcessingErr> {
        self.master.send_message(MasterMessage::SystemStop)?;
        self.send_status()?;

        Ok(())
    }

    fn stop(&mut self) -> Result<(), ActorProcessingErr> {
        self.controller
            .send_message(ControllerMessage::Forward(Packet::ActuatorStop))?;

        self.status = ActuatorStatus::Idle;
        self.send_status()?;

        Ok(())
    }
}
