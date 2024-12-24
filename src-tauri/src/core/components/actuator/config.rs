use std::sync::Arc;

use crate::{
    core::store::{PIDSettings, Store, StoreKey},
    utils::error::Error,
};

pub struct ActuatorConfig {
    pub precision: f64,
    pub scale_gain: f64,
    pub scale_offset: f64,
    pub pid_settings: PIDSettings,
    pub max_load: f64,
    pub min_load: f64,
}

impl TryFrom<Arc<Store>> for ActuatorConfig {
    type Error = Error;

    fn try_from(store: Arc<Store>) -> Result<Self, Self::Error> {
        let pid_settings = store
            .get(StoreKey::ActuatorPidSettings)
            .ok_or(Error::Config)?;

        let settings: PIDSettings = serde_json::from_value(pid_settings)?;

        let scale_gain = store.get(StoreKey::ScaleGain).ok_or(Error::Config)?;
        let scale_offset = store.get(StoreKey::ScaleOffset).ok_or(Error::Config)?;

        let max_load = store.get(StoreKey::ActuatorMaxLoad).ok_or(Error::Config)?;
        let min_load = store.get(StoreKey::ActuatorMinLoad).ok_or(Error::Config)?;
        let precision = store
            .get(StoreKey::ActuatorPrecision)
            .ok_or(Error::Config)?;

        Ok(Self {
            precision: precision.as_f64().ok_or(Error::InvalidStore)?,
            scale_gain: scale_gain.as_f64().ok_or(Error::InvalidStore)?,
            max_load: max_load.as_f64().ok_or(Error::InvalidStore)?,
            min_load: min_load.as_f64().ok_or(Error::InvalidStore)?,
            scale_offset: scale_offset.as_f64().ok_or(Error::InvalidStore)?,
            pid_settings: settings,
        })
    }
}
