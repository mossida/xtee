use std::{sync::Arc, time::Duration};

use serde::{Deserialize, Serialize};

use specta::Type;
use tauri::{AppHandle, Wry};
use tauri_plugin_store::{Error, StoreExt};

use crate::core::components::controller::Controller;

pub type Store = tauri_plugin_store::Store<Wry>;

#[derive(Serialize, Deserialize, Type)]
pub enum StoreKey {
    #[serde(rename = "scale.gain")]
    ScaleGain,
    #[serde(rename = "scale.offset")]
    ScaleOffset,
    #[serde(rename = "controllers.spawn")]
    Controllers,
    #[serde(rename = "actuator.tuning.setpoint")]
    ActuatorTuningSetpoint,
    #[serde(rename = "actuator.tuning.relay-amplitude")]
    ActuatorTuningRelayAmplitude,
    #[serde(rename = "actuator.pid.settings")]
    ActuatorPidSettings,
    #[serde(rename = "actuator.maxLoad")]
    ActuatorMaxLoad,
    #[serde(rename = "actuator.minLoad")]
    ActuatorMinLoad,
    #[serde(rename = "actuator.precision")]
    ActuatorPrecision,
    #[serde(rename = "motors.limits")]
    MotorsLimits,
    #[serde(rename = "motors.speeds")]
    MotorsSpeeds,
}

impl AsRef<str> for StoreKey {
    fn as_ref(&self) -> &str {
        match self {
            StoreKey::ScaleGain => "scale.gain",
            StoreKey::ScaleOffset => "scale.offset",
            StoreKey::Controllers => "controllers.spawn",
            StoreKey::ActuatorTuningSetpoint => "actuator.tuning.setpoint",
            StoreKey::ActuatorTuningRelayAmplitude => "actuator.tuning.relay-amplitude",
            StoreKey::ActuatorPidSettings => "actuator.pid.settings",
            StoreKey::ActuatorMaxLoad => "actuator.maxLoad",
            StoreKey::ActuatorMinLoad => "actuator.minLoad",
            StoreKey::ActuatorPrecision => "actuator.precision",
            StoreKey::MotorsLimits => "motors.limits",
            StoreKey::MotorsSpeeds => "motors.speeds",
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MotorsLimits {
    pub max_speed: u32,
    pub max_rotations: u32,
    pub acceleration: u32,
}

pub fn store(app: &AppHandle) -> Result<Arc<Store>, Error> {
    let builder = app
        .store_builder("store.json")
        .default(StoreKey::ScaleGain, 0.0000672315)
        .default(StoreKey::ScaleOffset, 0.0)
        .default(
            StoreKey::ActuatorPidSettings,
            serde_json::to_value(PIDSettings {
                proportional: 1.0,
                integral: 0.0,
                derivative: 0.0,
            })?,
        )
        .default(StoreKey::ActuatorMaxLoad, 200.0)
        .default(StoreKey::ActuatorMinLoad, 0.0)
        .default(StoreKey::ActuatorTuningSetpoint, 100.0)
        .default(StoreKey::ActuatorTuningRelayAmplitude, 100.0)
        .default(StoreKey::ActuatorPrecision, 1.0)
        .default(
            StoreKey::MotorsLimits,
            serde_json::to_value(MotorsLimits {
                max_speed: 1,
                max_rotations: 1,
                acceleration: 1,
            })?,
        )
        .default(
            StoreKey::Controllers,
            serde_json::to_value(vec![] as Vec<Controller>)?,
        )
        .auto_save(Duration::ZERO);

    builder.build()
}
