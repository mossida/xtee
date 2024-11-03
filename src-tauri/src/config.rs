use std::sync::Arc;

use serde_json::{Number, Value};

use crate::store::Store;

pub const FORWARD_PIN: &str = "forward_pin";
pub const BACKWARD_PIN: &str = "backward_pin";

pub const PID_PROPORTIONAL: &str = "pid_proportional";
pub const PID_INTEGRAL: &str = "pid_integral";
pub const PID_DERIVATIVE: &str = "pid_derivative";

pub fn defaults(store: Arc<Store>) {
    store.set(
        PID_PROPORTIONAL,
        Value::Number(Number::from_f64(1.0).unwrap()),
    );
    store.set(PID_INTEGRAL, Value::Number(Number::from_f64(0.0).unwrap()));
    store.set(
        PID_DERIVATIVE,
        Value::Number(Number::from_f64(0.0).unwrap()),
    );
}
