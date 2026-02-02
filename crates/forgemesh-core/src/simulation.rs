use rand::Rng;
use std::f64::consts::PI;

/// Simulates realistic industrial sensor behavior.
/// Generates physically plausible data with noise, cycles, and drift patterns.
/// 
/// # Usage
/// Used to generate "demo data" so the dashboard isn't empty during development
/// and for load testing the Merkle DAG storage under realistic write patterns.
pub struct IndustrialSimulator {
    base_value: f64,
    noise_level: f64,
    phase: f64,
    frequency: f64,
    drift: f64,
    spike_probability: f64,
}

impl IndustrialSimulator {
    /// Create temperature sensor simulator with daily cycle pattern
    /// Simulates HVAC or furnace temperature with day/night variation
    pub fn new_temperature(avg_temp: f64) -> Self {
        Self {
            base_value: avg_temp,
            noise_level: 0.5,      // +/- 0.5 degrees random noise
            phase: 0.0,
            frequency: 0.01,       // Slow sine wave (100 samples per cycle)
            drift: 0.0,
            spike_probability: 0.001, // 0.1% chance of sensor spike
        }
    }

    /// Create pressure sensor with pump-cycle oscillation
    /// Simulates hydraulic or pneumatic systems
    pub fn new_pressure(base_psi: f64) -> Self {
        Self {
            base_value: base_psi,
            noise_level: 2.0,
            phase: 0.0,
            frequency: 0.1,        // Faster oscillation (10 samples per cycle)
            drift: 0.0,
            spike_probability: 0.0,
        }
    }

    /// Create vibration sensor with bearing wear simulation
    /// Amplitude increases over time (drift) to simulate mechanical degradation
    pub fn new_vibration(base_g: f64) -> Self {
        Self {
            base_value: base_g,
            noise_level: 0.05,
            phase: 0.0,
            frequency: 0.6,        // ~60Hz equivalent sampling
            drift: 0.001,          // Gradual amplitude increase
            spike_probability: 0.02, // 2% chance of impact/vibration spike
        }
    }

    /// Generates next value in the sequence
    /// 
    /// Algorithm: Sine wave + Random Walk Noise + Occasional Spikes
    /// Returns value rounded to 2 decimal places for industrial precision
    pub fn next(&mut self) -> f64 {
        let mut rng = rand::thread_rng();
        let noise = rng.gen_range(-self.noise_level..self.noise_level);
        
        // Deterministic sine wave component (physical process)
        let cyclic = (self.phase * 2.0 * PI).sin() * 5.0;
        
        // Random spike injection (equipment faults/sensor errors)
        let spike = if rng.gen::<f64>() < self.spike_probability {
            rng.gen_range(10.0..25.0) // Jump 10-25 units
        } else {
            0.0
        };
        
        // Calculate value with all components
        let value = self.base_value + cyclic + noise + spike;
        
        // Update phase for next cycle
        self.phase += self.frequency;
        if self.phase > 1.0 { 
            self.phase -= 1.0; // Wrap around
        }
        
        // Apply drift (bearing wear, etc.)
        self.base_value += self.drift;
        
        // Round to 2 decimal places (industrial sensor precision)
        (value * 100.0).round() / 100.0
    }

    /// Generate batch of values efficiently
    /// Used for historical backfill without per-alloc overhead
    pub fn generate_batch(&mut self, count: usize) -> Vec<f64> {
        (0..count).map(|_| self.next()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_range() {
        let mut sim = IndustrialSimulator::new_temperature(25.0);
        
        // Generate 1000 samples, ensure physical plausibility
        // Allow for: base (25) + sine wave (±5) + noise (±0.5) + spike (up to +25)
        // Max: 25 + 5 + 0.5 + 25 = 55.5, Min: 25 - 5 - 0.5 = 19.5
        for _ in 0..1000 {
            let val = sim.next();
            assert!(val > 18.0 && val < 60.0, "Temperature {} out of realistic range", val);
        }
    }

    #[test]
    fn test_vibration_drift() {
        let mut sim = IndustrialSimulator::new_vibration(0.5);
        let initial_base = sim.base_value;
        
        // Run 1000 cycles
        for _ in 0..1000 {
            let _ = sim.next();
        }
        
        // Base should have drifted upward (bearing wear)
        assert!(sim.base_value > initial_base, "Bearing wear drift not applied");
    }

    #[test]
    fn test_batch_generation() {
        let mut sim = IndustrialSimulator::new_pressure(100.0);
        let batch = sim.generate_batch(100);
        assert_eq!(batch.len(), 100);
        // Verify all values are reasonable
        assert!(batch.iter().all(|&v| v > 80.0 && v < 120.0));
    }
}
