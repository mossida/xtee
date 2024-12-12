use std::{sync::Arc, time::Duration};

use serde::{Deserialize, Serialize};

use specta::Type;
use tauri::{AppHandle, Wry};
use tauri_plugin_store::{Error, StoreExt};

use crate::components::controller::Controller;

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
    #[serde(rename = "actuator.tuning.relay-amplitude")]
    ActuatorTuningRelayAmplitude,
}

impl AsRef<str> for StoreKey {
    fn as_ref(&self) -> &str {
        match self {
            StoreKey::ScaleGain => "scale.gain",
            StoreKey::ActuatorPidSettings => "actuator.pid.settings",
            StoreKey::Controllers => "controllers",
            StoreKey::ActuatorTuningSetpoint => "actuator.tuning.setpoint",
            StoreKey::ActuatorTuningRelayAmplitude => "actuator.tuning.relay-amplitude",
        }
    }
}

impl From<StoreKey> for String {
    fn from(val: StoreKey) -> Self {
        val.as_ref().to_string()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PIDSettings {
    pub proportional: f32,
    pub integral: f32,
    pub derivative: f32,
}

pub fn store(app: &AppHandle) -> Result<Arc<Store>, Error> {
    let builder = app
        .store_builder("store.json")
        .default(StoreKey::ScaleGain, 0.0000672315)
        .default(
            StoreKey::ActuatorPidSettings,
            serde_json::to_value(PIDSettings {
                proportional: 1.2,
                integral: 0.04,
                derivative: 0.15,
            })?,
        )
        .default(StoreKey::ActuatorTuningSetpoint, 100.0)
        .default(StoreKey::ActuatorTuningRelayAmplitude, 100.0)
        .default(
            StoreKey::Controllers,
            serde_json::to_value(vec![] as Vec<Controller>)?,
        )
        .auto_save(Duration::from_millis(100));

    builder.build()
}
