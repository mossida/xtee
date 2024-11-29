/// A simple 1D Kalman filter implementation for smoothing weight measurements
#[derive(Debug, Clone)]
pub struct KalmanFilter {
    // State estimate
    estimate: f32,
    // Estimate uncertainty
    estimate_uncertainty: f32,
    // Measurement uncertainty
    measurement_uncertainty: f32,
    // Process noise
    process_noise: f32,
}

impl KalmanFilter {
    /// Creates a new KalmanFilter instance
    ///
    /// # Arguments
    /// * `initial_estimate` - Initial state estimate
    /// * `estimate_uncertainty` - Initial estimate uncertainty
    /// * `measurement_uncertainty` - Measurement uncertainty (noise in the sensor)
    /// * `process_noise` - Process noise (how much the true value can change between measurements)
    pub fn new(
        initial_estimate: f32,
        estimate_uncertainty: f32,
        measurement_uncertainty: f32,
        process_noise: f32,
    ) -> Self {
        Self {
            estimate: initial_estimate,
            estimate_uncertainty,
            measurement_uncertainty,
            process_noise,
        }
    }

    /// Updates the filter with a new measurement and returns the filtered value
    ///
    /// # Arguments
    /// * `measurement` - The new measurement to process
    pub fn update(&mut self, measurement: f32) -> f32 {
        // Prediction step
        let predicted_uncertainty = self.estimate_uncertainty + self.process_noise;

        // Update step
        // Calculate Kalman gain
        let kalman_gain =
            predicted_uncertainty / (predicted_uncertainty + self.measurement_uncertainty);

        // Update estimate
        self.estimate = self.estimate + kalman_gain * (measurement - self.estimate);

        // Update estimate uncertainty
        self.estimate_uncertainty = (1.0 - kalman_gain) * predicted_uncertainty;

        self.estimate
    }
}
