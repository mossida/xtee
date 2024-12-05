use std::{sync::Arc, time::Duration};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Wry};
use tauri_plugin_store::{Error, StoreExt};

use crate::components::controller::{Controller, ControllerGroup};

pub const CONTROLLERS: &str = "controllers";
pub const PID_SETTINGS: &str = "pid_settings";
pub const SCALE_GAIN: &str = "scale_gain";

pub type Store = tauri_plugin_store::Store<Wry>;

#[derive(Serialize, Deserialize)]
pub struct PIDSettings {
    pub proportional: f32,
    pub integral: f32,
    pub derivative: f32,
    pub proportional_limit: f32,
    pub integral_limit: f32,
    pub derivative_limit: f32,
}

fn defaults(store: Arc<Store>) {
    store.set(
        PID_SETTINGS,
        serde_json::to_value(PIDSettings {
            proportional: 1.2,
            integral: 0.04,
            derivative: 0.15,
            proportional_limit: 2.0,
            integral_limit: 0.05,
            derivative_limit: 0.2,
        })
        .unwrap(),
    );

    store.set(SCALE_GAIN, 0.0000672315);
    store.set(
        CONTROLLERS,
        serde_json::to_value(vec![
            Controller {
                id: "controller-motors".to_owned(),
                group: ControllerGroup::Motors,
                serial_port: "/dev/cu.usbmodem113201".to_owned(),
                baud_rate: 115200,
            },
            /*Controller {
                id: "controller-motors".to_owned(),
                group: ControllerGroup::Motors,
                serial_port: "/dev/cu.usbmodem113301".to_owned(),
                baud_rate: 115200,
            },*/
        ] as Vec<Controller>)
        .unwrap(),
    );
}

pub fn store(app: &AppHandle) -> Result<Arc<Store>, Error> {
    let store = app
        .store_builder("store.json")
        .auto_save(Duration::from_millis(100))
        .build()?;

    defaults(store.clone());

    Ok(store)
}
