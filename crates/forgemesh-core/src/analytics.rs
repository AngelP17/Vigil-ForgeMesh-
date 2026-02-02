use crate::types::DataNode;
use serde::Serialize;

/// Edge-computed statistics for sensor data
/// Calculates min/max/avg locally to reduce bandwidth to cloud
#[derive(Debug, Serialize, Default, Clone)]
pub struct SensorStats {
    pub count: usize,
    pub min: f64,
    pub max: f64,
    pub avg: f64,
    pub last_value: f64,
    pub last_seen: u64,
    pub variance: f64,      // NEW: Standard deviation squared
    pub std_dev: f64,       // NEW: For anomaly detection
}

impl SensorStats {
    /// Compute statistics from history slice
    /// O(n) time, O(1) space - efficient for edge devices
    pub fn compute(history: &[DataNode]) -> Self {
        if history.is_empty() {
            return SensorStats::default();
        }

        let mut stats = SensorStats {
            count: history.len(),
            min: f64::MAX,
            max: f64::MIN,
            avg: 0.0,
            last_value: history[0].value,
            last_seen: history[0].timestamp_ns,
            variance: 0.0,
            std_dev: 0.0,
        };

        // Single-pass algorithm for mean and variance (Welford's algorithm)
        // More numerically stable than sum/count for large datasets
        let mut mean = 0.0;
        let mut m2 = 0.0; // Sum of squares of differences
        
        for (i, node) in history.iter().enumerate() {
            // Min/Max tracking
            if node.value < stats.min { stats.min = node.value; }
            if node.value > stats.max { stats.max = node.value; }
            
            // Welford's online algorithm
            let x = node.value;
            let delta = x - mean;
            mean += delta / (i + 1) as f64;
            let delta2 = x - mean;
            m2 += delta * delta2;
        }
        
        stats.avg = (mean * 100.0).round() / 100.0;
        
        // Sample variance (using n-1 denominator for unbiased estimate)
        if history.len() > 1 {
            stats.variance = m2 / (history.len() - 1) as f64;
            stats.std_dev = stats.variance.sqrt();
        }
        
        stats
    }

    /// Detect if value is anomalous (超出3-sigma范围)
    /// Returns true if value > 3 standard deviations from mean
    pub fn is_anomaly(&self, value: f64) -> bool {
        if self.count < 10 || self.std_dev == 0.0 {
            return false; // Not enough data
        }
        let z_score = (value - self.avg).abs() / self.std_dev;
        z_score > 3.0
    }

    /// Get trend direction based on first vs last value
    pub fn trend(&self, history: &[DataNode]) -> TrendDirection {
        if history.len() < 2 {
            return TrendDirection::Stable;
        }
        let first = history.last().unwrap().value; // Oldest (end of vec)
        let last = history.first().unwrap().value; // Newest (start of vec)
        
        let delta = ((last - first) / first.abs()).abs();
        if delta < 0.01 {
            TrendDirection::Stable
        } else if last > first {
            TrendDirection::Rising
        } else {
            TrendDirection::Falling
        }
    }
}

#[derive(Debug, Serialize)]
pub enum TrendDirection {
    Rising,
    Falling,
    Stable,
}

/// OEE (Overall Equipment Effectiveness) Metrics
#[derive(Debug, Serialize, Default)]
pub struct OEEMetrics {
    pub availability: f64,    // Time machine was running
    pub performance: f64,     // Speed vs theoretical max
    pub quality: f64,         // Good parts / total parts
    pub oee: f64,             // A * P * Q
    pub runtime_mins: u64,
    pub downtime_mins: u64,
}

impl OEEMetrics {
    /// Calculate OEE from operational parameters
    pub fn calculate(
        runtime_mins: u64, 
        planned_mins: u64,
        actual_count: u32,
        theoretical_rate: f64, // units per minute
        good_count: u32
    ) -> Self {
        let availability = (runtime_mins as f64 / planned_mins as f64).min(1.0);
        let potential_output = theoretical_rate * runtime_mins as f64;
        let performance = (actual_count as f64 / potential_output).min(1.0);
        let quality = if actual_count > 0 {
            good_count as f64 / actual_count as f64
        } else {
            1.0
        };
        
        Self {
            availability: (availability * 100.0).round() / 100.0,
            performance: (performance * 100.0).round() / 100.0,
            quality: (quality * 100.0).round() / 100.0,
            oee: ((availability * performance * quality) * 100.0).round() / 100.0,
            runtime_mins,
            downtime_mins: planned_mins - runtime_mins,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DataNode;

    fn create_test_node(value: f64, ts: u64) -> DataNode {
        DataNode::new("test", value, ts, None)
    }

    #[test]
    fn test_stats_calculation() {
        let history = vec![
            create_test_node(10.0, 1),
            create_test_node(20.0, 2),
            create_test_node(30.0, 3),
        ];
        
        let stats = SensorStats::compute(&history);
        assert_eq!(stats.count, 3);
        assert_eq!(stats.min, 10.0);
        assert_eq!(stats.max, 30.0);
        assert_eq!(stats.avg, 20.0);
    }

    #[test]
    fn test_anomaly_detection() {
        let history: Vec<_> = (0..100)
            .map(|i| create_test_node(50.0 + (i as f64 * 0.1), i))
            .collect();
        
        let stats = SensorStats::compute(&history);
        assert!(!stats.is_anomaly(50.0));       // Normal value
        assert!(stats.is_anomaly(1000.0));      // Clear anomaly
    }

    #[test]
    fn test_oee_calculation() {
        // 8 hour shift, 30 min downtime, 450 units produced, 445 good
        let oee = OEEMetrics::calculate(450, 480, 450, 1.0, 445);
        
        assert_eq!(oee.availability, 0.94); // 450/480
        assert_eq!(oee.performance, 1.0);   // 450/450
        assert!(oee.quality > 0.98);        // 445/450 = 0.988...
        assert!(oee.oee > 0.0 && oee.oee <= 1.0);
    }
}
