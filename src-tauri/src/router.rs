use rspc::{Config, Router};
use tauri::AppHandle;

use crate::cmd::{
    actuator_keep, actuator_load, actuator_move, actuator_reload_settings, actuator_stop,
    actuator_tune, events, get_controllers, get_groups, get_ports, motor_get_max_speed, motor_keep,
    motor_set_outputs, motor_spin, motor_stop, restart, spawn_controller,
};

pub struct RouterContext {
    #[allow(unused)]
    pub handle: AppHandle,
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
        .mutation("restart", |t| t(restart))
        .mutation("motor/keep", |t| t(motor_keep))
        .mutation("motor/spin", |t| t(motor_spin))
        .mutation("motor/stop", |t| t(motor_stop))
        .mutation("motor/set/outputs", |t| t(motor_set_outputs))
        .mutation("actuator/move", |t| t(actuator_move))
        .mutation("actuator/stop", |t| t(actuator_stop))
        .mutation("actuator/load", |t| t(actuator_load))
        .mutation("actuator/keep", |t| t(actuator_keep))
        .mutation("actuator/tune", |t| t(actuator_tune))
        .mutation("actuator/reload/settings", |t| t(actuator_reload_settings))
        .build()
}
