use std::{sync::Arc, time::Duration};

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::{AppHandle, Wry};
use tauri_plugin_store::{Error, StoreExt};

use crate::components::controller::{Controller, ControllerGroup};

pub type Store = tauri_plugin_store::Store<Wry>;

#[derive(Serialize, Deserialize, Type)]
pub enum StoreKey {
    #[serde(rename = "scale.gain")]
    ScaleGain,
    #[serde(rename = "actuator.pid.settings")]
    ActuatorPidSettings,
    #[serde(rename = "controllers")]
    Controllers,
    #[serde(rename = "actuator.tuning.setpoint")]
    ActuatorTuningSetpoint,
}

impl AsRef<str> for StoreKey {
    fn as_ref(&self) -> &str {
        match self {
            StoreKey::ScaleGain => "scale.gain",
            StoreKey::ActuatorPidSettings => "actuator.pid.settings",
            StoreKey::Controllers => "controllers",
            StoreKey::ActuatorTuningSetpoint => "actuator.tuning.setpoint",
        }
    }
}

impl Into<String> for StoreKey {
    fn into(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

#[derive(Serialize, Deserialize)]
pub struct PIDSettings {
    pub proportional: f32,
    pub integral: f32,
    pub derivative: f32,
}

fn defaults(store: Arc<Store>) {
    store.set(
        StoreKey::ActuatorPidSettings,
        serde_json::to_value(PIDSettings {
            proportional: 1.2,
            integral: 0.04,
            derivative: 0.15,
        })
        .unwrap(),
    );

    store.set(StoreKey::ActuatorTuningSetpoint, 100.0);

    store.set(StoreKey::ScaleGain, 0.0000672315);
    store.set(
        StoreKey::Controllers,
        serde_json::to_value(vec![
            Controller {
                id: "controller-motors".to_owned(),
                group: ControllerGroup::Motors,
                serial_port: "/dev/tty.usbmodem113201".to_owned(),
                baud_rate: 115200,
            },
            Controller {
                id: "controller-default".to_owned(),
                group: ControllerGroup::Default,
                serial_port: "/dev/tty.usbserial-11330".to_owned(),
                baud_rate: 115200,
            },
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
