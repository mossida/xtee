use std::{sync::Arc, time::Duration};

use tauri::{AppHandle, Wry};
use tauri_plugin_store::{Error, StoreExt};

pub const CONTROLLER_BUS: &str = "controller_bus";
pub const CONTROLLER_BAUD: &str = "controller_baud";

pub const PID_PROPORTIONAL: &str = "pid_proportional";
pub const PID_INTEGRAL: &str = "pid_integral";
pub const PID_DERIVATIVE: &str = "pid_derivative";

pub const SCALE_GAIN: &str = "scale_gain";

pub type Store = tauri_plugin_store::Store<Wry>;

fn defaults(store: Arc<Store>) {
    store.set(PID_PROPORTIONAL, 1.2);
    store.set(PID_INTEGRAL, 0.04);
    store.set(PID_DERIVATIVE, 0.15);

    store.set(SCALE_GAIN, 0.0000672315);

    store.set(CONTROLLER_BUS, "/dev/cu.usbmodem113101");
    store.set(CONTROLLER_BAUD, 115200);
}

pub fn store(app: &AppHandle) -> Result<Arc<Store>, Error> {
    let store = app
        .store_builder("store.json")
        .auto_save(Duration::from_millis(100))
        .build()?;

    defaults(store.clone());

    Ok(store)
}
