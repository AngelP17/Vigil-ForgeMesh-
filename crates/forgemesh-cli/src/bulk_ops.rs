use forgemesh_core::{store::ForgeStore, simulation::IndustrialSimulator};
use indicatif::{ProgressBar, ProgressStyle};
use anyhow::Result;
use std::time::{SystemTime, UNIX_EPOCH};

/// Generate historical data leading up to now
/// Creates realistic backfill for dashboard demonstration
pub fn generate_history(
    store: &ForgeStore, 
    sensor: &str, 
    count: usize,
    sensor_type: &str
) -> Result<()> {
    let mut sim = match sensor_type {
        "pressure" => IndustrialSimulator::new_pressure(100.0),
        "vibration" => IndustrialSimulator::new_vibration(0.5),
        _ => IndustrialSimulator::new_temperature(22.5),
    };
    
    let pb = ProgressBar::new(count as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")?
            .progress_chars("#>-")
    );

    // Calculate start time (count seconds ago)
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos() as u64;
    let start_time = now.saturating_sub(count as u64 * 1_000_000_000); // 1 second intervals

    for i in 0..count {
        let val = sim.next();
        let ts = start_time + (i as u64 * 1_000_000_000);
        store.put(sensor, val, ts)?;
        
        if i % 100 == 0 {
            pb.set_position(i as u64);
            pb.set_message(format!("{}: {:.2}", sensor, val));
        }
    }

    pb.finish_with_message("Generation Complete");
    Ok(())
}

/// Simulate complete production line with multiple sensors
pub fn simulate_line(store: &ForgeStore, line_id: &str) -> Result<()> {
    println!("Generating production line data for: {}", line_id);
    
    let sensors = vec![
        (format!("{}-temp", line_id), "temperature", 10080), // 1 week of minute data
        (format!("{}-pressure", line_id), "pressure", 10080),
        (format!("{}-vibration", line_id), "vibration", 10080),
    ];
    
    for (sensor_id, sensor_type, count) in sensors {
        println!("Populating {} with {} historical points...", sensor_id, count);
        generate_history(store, &sensor_id, count, sensor_type)?;
    }
    
    println!("✓ Production line '{}' simulation complete", line_id);
    Ok(())
}
