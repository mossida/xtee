use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    core::store::{Store, StoreKey},
    utils::error::Error,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MotorsLimits {
    pub max_speed: u32,
    pub max_rotations: u32,
    pub acceleration: u32,
    pub steps_per_pulse: u16,
}

impl TryFrom<Arc<Store>> for MotorsLimits {
    type Error = Error;

    fn try_from(store: Arc<Store>) -> Result<Self, Error> {
        let limits_value = store.get(StoreKey::MotorsLimits).ok_or(Error::Config)?;
        let limits: MotorsLimits = serde_json::from_value(limits_value)?;

        Ok(limits)
    }
}
