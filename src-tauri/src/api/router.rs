use ractor::ActorRef;
use rspc::{Config, Router};

use crate::{
    api::cmd::{
        actuator_keep, actuator_load, actuator_move, actuator_reload_settings, actuator_stop,
        events, get_controllers, get_groups, get_ports, kill_controller, motor_get_max_speed,
        motor_keep, motor_reload_settings, motor_set_outputs, motor_spin, motor_stop,
        spawn_controller,
    },
    core::components::master::MasterMessage,
};

pub struct RouterContext {
    pub master: ActorRef<MasterMessage>,
}

pub fn router() -> Router<RouterContext> {
    Router::new()
        .config(Config::new().export_ts_bindings("../src/types/bindings.ts"))
        .query("master/events", |t| t(events))
        .query("master/groups", |t| t(get_groups))
        .query("master/controllers", |t| t(get_controllers))
        .query("master/ports", |t| t(get_ports))
        .query("motor/get/max-speed", |t| t(motor_get_max_speed))
        .mutation("master/spawn", |t| t(spawn_controller))
        .mutation("master/kill", |t| t(kill_controller))
        .mutation("motor/keep", |t| t(motor_keep))
        .mutation("motor/spin", |t| t(motor_spin))
        .mutation("motor/stop", |t| t(motor_stop))
        .mutation("motor/set/outputs", |t| t(motor_set_outputs))
        .mutation("motor/reload/settings", |t| t(motor_reload_settings))
        .mutation("actuator/move", |t| t(actuator_move))
        .mutation("actuator/stop", |t| t(actuator_stop))
        .mutation("actuator/load", |t| t(actuator_load))
        .mutation("actuator/keep", |t| t(actuator_keep))
        .mutation("actuator/reload/settings", |t| t(actuator_reload_settings))
        .build()
}
