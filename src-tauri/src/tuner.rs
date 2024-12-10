use tokio::time::Instant;

#[derive(Debug)]
pub struct PIDParameters {
    pub kp: f32,
    pub ki: f32,
    pub kd: f32,
}

pub struct Tuner {
    relay_amplitude_ms: f32, // Relay amplitude in milliseconds
    setpoint_kg: f32,        // Setpoint in kilograms
    critical_gain: f32,
    critical_period: f32,
    oscillation_start: Option<Instant>,
    last_crossing: Option<Instant>,
    last_value: Option<f32>,
    oscillation_count: u32,
    min_oscillations: u32,
    is_tuning_complete: bool,
    peak_to_peak: f32,
    is_preload_verified: bool,
}

impl Tuner {
    /// Creates a new tuner
    /// * `relay_amplitude_ms` - The amplitude of the relay output in milliseconds
    /// * `setpoint_kg` - The target force in kilograms around which to oscillate
    pub fn new(relay_amplitude_ms: f32, setpoint_kg: f32) -> Self {
        assert!(
            setpoint_kg > 0.1,
            "Setpoint must be greater than 0.1 kg for safety"
        );
        assert!(
            relay_amplitude_ms > 1.0 && relay_amplitude_ms < 1000.0,
            "Relay amplitude must be between 1 and 1000 milliseconds"
        );

        Self {
            relay_amplitude_ms,
            setpoint_kg,
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

    /// Verify if the initial tension is close enough to the setpoint
    /// Returns true if the preload is acceptable
    pub fn verify_preload(&mut self, current_kg: f32) -> bool {
        let error_kg = (self.setpoint_kg - current_kg).abs();
        let acceptable_error = self.setpoint_kg * 0.15; // 15% tolerance

        self.is_preload_verified = error_kg <= acceptable_error;
        self.is_preload_verified
    }

    /// Process a measurement and return the actuation time in milliseconds
    pub fn process_measurement(&mut self, current_kg: f32) -> f32 {
        // Safety check for preload
        if !self.is_preload_verified {
            if !self.verify_preload(current_kg) {
                return 0.0; // Don't actuate if preload isn't verified
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
                // Setpoint crossing detected
                let now = Instant::now();

                if let Some(last_crossing) = self.last_crossing {
                    // Calculate period between crossings
                    let half_period = now.duration_since(last_crossing).as_secs_f32();
                    self.critical_period = half_period * 2.0;

                    // Update peak-to-peak amplitude
                    self.peak_to_peak = self.peak_to_peak.max((current_kg - last_value).abs());

                    // Increment oscillation count
                    self.oscillation_count += 1;

                    // Check if we have enough oscillations
                    if self.oscillation_count >= self.min_oscillations {
                        // Calculate critical gain using relay feedback method
                        // Kc = (4 * relay_amplitude) / (π * oscillation_amplitude)
                        self.critical_gain = (4.0 * self.relay_amplitude_ms)
                            / (std::f32::consts::PI * self.peak_to_peak);
                        self.is_tuning_complete = true;
                        return 0.0;
                    }
                }
                self.last_crossing = Some(now);
            }
        }

        self.last_value = Some(current_kg);

        // Return actuation time in milliseconds
        if error > 0.0 {
            self.relay_amplitude_ms // Pull harder
        } else {
            -self.relay_amplitude_ms // Release
        }
    }

    pub fn is_tuning_complete(&self) -> bool {
        self.is_tuning_complete
    }

    pub fn is_preload_ok(&self) -> bool {
        self.is_preload_verified
    }

    /// Get PID parameters (output will be in milliseconds per kg error)
    pub fn get_pid_parameters(&self) -> Option<PIDParameters> {
        if !self.is_tuning_complete {
            return None;
        }

        // Calculate PID parameters using Ziegler-Nichols rules
        Some(PIDParameters {
            kp: 0.6 * self.critical_gain,
            ki: 1.2 * self.critical_gain / self.critical_period,
            kd: 0.075 * self.critical_gain * self.critical_period,
        })
    }

    /// Get PI parameters (output will be in milliseconds per kg error)
    pub fn get_pi_parameters(&self) -> Option<PIDParameters> {
        if !self.is_tuning_complete {
            return None;
        }

        Some(PIDParameters {
            kp: 0.45 * self.critical_gain,
            ki: 0.54 * self.critical_gain / self.critical_period,
            kd: 0.0,
        })
    }

    /// Get P parameters (output will be in milliseconds per kg error)
    pub fn get_p_parameters(&self) -> Option<PIDParameters> {
        if !self.is_tuning_complete {
            return None;
        }

        Some(PIDParameters {
            kp: 0.5 * self.critical_gain,
            ki: 0.0,
            kd: 0.0,
        })
    }

    pub fn get_tuning_metrics(&self) -> Option<(f32, f32)> {
        if !self.is_tuning_complete {
            return None;
        }
        Some((self.critical_gain, self.critical_period))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tuner_initialization() {
        let tuner = Tuner::new(100.0, 5.0);
        assert!(!tuner.is_tuning_complete());
    }

    #[test]
    #[should_panic(expected = "Setpoint must be greater than 0.1 kg")]
    fn test_invalid_setpoint() {
        Tuner::new(100.0, 0.05);
    }

    #[test]
    fn test_preload_verification() {
        let mut tuner = Tuner::new(100.0, 5.0);

        // Test within 15% tolerance
        assert!(tuner.verify_preload(4.5)); // 10% below setpoint
        assert!(tuner.verify_preload(5.5)); // 10% above setpoint

        // Test outside tolerance
        assert!(!tuner.verify_preload(4.0)); // 20% below setpoint
        assert!(!tuner.verify_preload(6.0)); // 20% above setpoint
    }

    #[test]
    fn test_relay_output() {
        let mut tuner = Tuner::new(100.0, 5.0);

        // First verify preload
        assert!(tuner.verify_preload(5.1));

        // Test relay switching around setpoint
        let output1 = tuner.process_measurement(5.5); // Above setpoint
        assert_eq!(output1, -100.0); // Should release

        let output2 = tuner.process_measurement(4.5); // Below setpoint
        assert_eq!(output2, 100.0); // Should pull
    }

    #[test]
    fn test_reset() {
        let mut tuner = Tuner::new(100.0, 5.0);

        // Simulate some tuning progress
        tuner.verify_preload(5.1);
        tuner.process_measurement(5.0);

        // Reset the tuner
        tuner.reset();

        // Verify reset state
        assert!(!tuner.is_tuning_complete());
        assert!(!tuner.is_preload_ok());
        assert_eq!(tuner.oscillation_count, 0);
    }
}
