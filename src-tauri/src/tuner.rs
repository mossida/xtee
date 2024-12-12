use tokio::time::Instant;
use tracing::info;

#[derive(Debug)]
pub struct PIDParameters {
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
}

#[derive(Debug)]
pub struct PIDLimits {
    pub output_limit: f64, // Overall output limit in ms
    pub p_limit: f64,      // P term limit in ms
    pub i_limit: f64,      // I term limit in ms
    pub d_limit: f64,      // D term limit in ms
}

pub struct Tuner {
    relay_amplitude_ms: f64, // Relay amplitude in milliseconds
    setpoint_kg: f64,        // Setpoint in kilograms
    critical_gain: f64,
    critical_period: f64,
    oscillation_start: Option<Instant>,
    last_crossing: Option<Instant>,
    last_value: Option<f64>,
    oscillation_count: u32,
    min_oscillations: u32,
    is_tuning_complete: bool,
    peak_to_peak: f64,
    is_preload_verified: bool,
}

impl Tuner {
    /// Creates a new tuner
    /// * `relay_amplitude_ms` - The amplitude of the relay output in milliseconds
    /// * `setpoint_kg` - The target force in kilograms around which to oscillate
    pub fn new() -> Self {
        Self {
            relay_amplitude_ms: 1.0,
            setpoint_kg: 0.1,
            critical_gain: 0.0,
            critical_period: 0.0,
            oscillation_start: None,
            last_crossing: None,
            last_value: None,
            oscillation_count: 0,
            min_oscillations: 4,
            is_tuning_complete: false,
            peak_to_peak: 0.0,
            is_preload_verified: false,
        }
    }

    pub fn set_setpoint(&mut self, setpoint_kg: f64) {
        assert!(
            setpoint_kg > 0.1,
            "Setpoint must be greater than 0.1 kg for safety"
        );

        self.setpoint_kg = setpoint_kg;
    }

    pub fn set_relay_amplitude(&mut self, relay_amplitude_ms: f64) {
        assert!(
            relay_amplitude_ms > 1.0 && relay_amplitude_ms < 1000.0,
            "Relay amplitude must be between 1 and 1000 milliseconds"
        );

        self.relay_amplitude_ms = relay_amplitude_ms;
    }

    /// Verify if the initial tension is close enough to the setpoint
    /// Returns true if the preload is acceptable
    pub fn verify_preload(&mut self, current_kg: f64) -> bool {
        let error_kg = (self.setpoint_kg - current_kg).abs();
        let acceptable_error = self.setpoint_kg * 0.15; // 15% tolerance

        self.is_preload_verified = error_kg <= acceptable_error;
        self.is_preload_verified
    }

    /// Process a measurement and return the actuation time in milliseconds
    pub fn process_measurement(&mut self, current_kg: f64) -> f64 {
        // Safety check for preload
        if !self.is_preload_verified {
            if !self.verify_preload(current_kg) {
                return 0.0;
            }
        }

        if self.is_tuning_complete {
            return 0.0;
        }

        let error = self.setpoint_kg - current_kg;

        // Initialize last_value if this is the first measurement
        if self.last_value.is_none() {
            self.last_value = Some(current_kg);
            self.oscillation_start = Some(Instant::now());
            return self.relay_amplitude_ms;
        }

        // Detect setpoint crossing
        if let Some(last_value) = self.last_value {
            let last_error = self.setpoint_kg - last_value;
            if error * last_error < 0.0 {
                let now = Instant::now();

                if let Some(last_crossing) = self.last_crossing {
                    let half_period = now.duration_since(last_crossing).as_secs_f64();
                    self.critical_period = half_period * 2.0;
                    self.peak_to_peak = self.peak_to_peak.max((current_kg - last_value).abs());
                    self.oscillation_count += 1;

                    if self.oscillation_count >= self.min_oscillations {
                        // Calculate critical gain in ms/kg
                        // relay_amplitude_ms is in ms, peak_to_peak is in kg
                        // So critical_gain will be in ms/kg
                        self.critical_gain = (4.0 * self.relay_amplitude_ms)
                            / (std::f64::consts::PI * self.peak_to_peak);
                        self.is_tuning_complete = true;
                        return 0.0;
                    }
                }
                self.last_crossing = Some(now);
            }
        }

        self.last_value = Some(current_kg);

        if error > 0.0 {
            self.relay_amplitude_ms
        } else {
            -self.relay_amplitude_ms
        }
    }

    pub fn is_tuning_complete(&self) -> bool {
        self.is_tuning_complete
    }

    pub fn is_preload_ok(&self) -> bool {
        self.is_preload_verified
    }

    /// Get PID parameters using Ziegler-Nichols rules scaled for practical actuation times
    pub fn get_pid_parameters(&self) -> Option<PIDParameters> {
        if !self.is_tuning_complete {
            return None;
        }

        info!(
            "Tuning results - Critical gain: {:.2} ms/kg, Period: {:.2}s, Peak-to-peak: {:.2}kg",
            self.critical_gain, self.critical_period, self.peak_to_peak,
        );

        Some(PIDParameters {
            kp: 0.6 * self.critical_gain,
            ki: 1.2 * self.critical_gain / self.critical_period,
            kd: 0.075 * self.critical_gain * self.critical_period,
        })
    }

    /// Reset the tuner to its initial state while keeping the same setpoint and relay amplitude
    pub fn reset(&mut self) {
        self.critical_gain = 0.0;
        self.critical_period = 0.0;
        self.oscillation_start = None;
        self.last_crossing = None;
        self.last_value = None;
        self.oscillation_count = 0;
        self.is_tuning_complete = false;
        self.peak_to_peak = 0.0;
        self.is_preload_verified = false;
    }
}
