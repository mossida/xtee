use rspc::{Config, Router};
use tauri::AppHandle;

use crate::cmd::{
    actuator_keep, actuator_load, actuator_move, actuator_reload_settings, actuator_stop,
    actuator_tune, events, get_controllers, motor_get_max_speed, motor_keep, motor_set_outputs,
    motor_spin, motor_stop, restart,
};

pub struct RouterContext {
    #[allow(unused)]
    pub handle: AppHandle,
}

pub fn router() -> Router<RouterContext> {
    Router::new()
        .config(Config::new().export_ts_bindings("../src/types/bindings.ts"))
        .query("events", |t| t(events))
        .query("controllers", |t| t(get_controllers))
        .query("motor/get/max-speed", |t| t(motor_get_max_speed))
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
