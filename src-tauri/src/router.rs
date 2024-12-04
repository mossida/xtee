use rspc::{Config, Router};
use tauri::AppHandle;

use crate::cmd::{
    actuator_keep, actuator_load, actuator_move, actuator_stop, get_controllers, get_ports,
    motor_spin, motor_stop, restart,
};

pub struct RouterContext {
    #[allow(unused)]
    pub handle: AppHandle,
}

pub fn router() -> Router<RouterContext> {
    Router::new()
        .config(Config::new().export_ts_bindings("../src/types/bindings.ts"))
        .query("controllers", |t| t(get_controllers))
        //.query("ports", |t| t(get_ports))
        .mutation("restart", |t| t(restart))
        .mutation("motor/spin", |t| t(motor_spin))
        .mutation("motor/stop", |t| t(motor_stop))
        .mutation("actuator/move", |t| t(actuator_move))
        .mutation("actuator/stop", |t| t(actuator_stop))
        .mutation("actuator/load", |t| t(actuator_load))
        .mutation("actuator/keep", |t| t(actuator_keep))
        .build()
}
