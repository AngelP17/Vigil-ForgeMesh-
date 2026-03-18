use anyhow::Result;
use clap::{Parser, Subcommand};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tracing::info;
use vigil_core::merkle;
use vigil_core::store::ForgeStore;
use vigil_core::{init_sqlite_pool, run_incident_pipeline};
use vigil_sync::car::CarExporter;

mod bulk_ops;

#[derive(Parser)]
#[command(name = "vigil")]
#[command(about = "Operational Incident Intelligence Platform with Merkle-backed replay")]
struct Cli {
    #[arg(long, default_value = "./vigil_data")]
    db_path: String,
    #[arg(long)]
    incident_db: Option<String>,
    #[arg(long, default_value = "node-local")]
    node_id: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Write {
        #[arg(short, long)]
        sensor: String,
        #[arg(short, long)]
        value: f64,
    },
    Log {
        #[arg(short, long)]
        sensor: String,
        #[arg(short = 'n', long, default_value_t = 10)]
        limit: usize,
    },
    Verify {
        #[arg(short, long)]
        sensor: String,
    },
    Bench {
        #[arg(long, default_value_t = 10000)]
        count: usize,
    },
    Export {
        #[arg(short, long)]
        output: String,
    },
    Import {
        #[arg(short, long)]
        file: String,
    },
    Daemon {
        #[arg(long, default_value_t = 8080)]
        port: u16,
    },
    /// Generate simulated historical data
    Generate {
        #[arg(short, long)]
        sensor: String,
        #[arg(short, long, default_value_t = 1000)]
        count: usize,
        #[arg(long, default_value = "temperature")]
        sensor_type: String,
    },
    /// Simulate complete production line
    SimulateLine {
        #[arg(short, long)]
        line: String,
    },
    /// Seed demo data into Sled, SQLite, and exported sample files
    SeedDemo,
    /// Run the incident pipeline against current telemetry and context
    Detect,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();
    let incident_db = cli
        .incident_db
        .clone()
        .unwrap_or_else(|| format!("{}/vigil.db", cli.db_path));

    match cli.command {
        Commands::Daemon { port } => {
            let store = Arc::new(Mutex::new(ForgeStore::new(&cli.db_path)?));
            let db = init_sqlite_pool(&incident_db).await?;
            {
                let store_guard = store.lock().await;
                vigil_core::simulation::seed_demo_environment(&db, &store_guard, "./data").await?;
                let summary = run_incident_pipeline(&db, &store_guard).await?;
                info!(
                    "Incident pipeline initialized: {} incidents from {} events",
                    summary.created_incident_ids.len(),
                    summary.events_processed
                );
            }
            let (tx, _rx) = tokio::sync::broadcast::channel(256);
            let state = Arc::new(vigil_web::AppState {
                store: store.clone(),
                db,
                node_id: cli.node_id.clone(),
                tx,
            });
            let app = vigil_web::create_router(state);
            let addr = format!("0.0.0.0:{}", port);
            info!("Daemon starting on http://{}", addr);
            println!("Vigil UI: http://{}:{}", "localhost", port);
            axum::serve(tokio::net::TcpListener::bind(&addr).await?, app).await?;
        }
        cmd => {
            let store = ForgeStore::new(&cli.db_path)?;
            match cmd {
                Commands::Write { sensor, value } => {
                    let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos() as u64;
                    let hash = store.put(&sensor, value, ts)?;
                    println!("✓ Written {} = {} (hash {}...)", sensor, value, &hash[..16]);
                }
                Commands::Log { sensor, limit } => {
                    let hist = store.get_history(&sensor, limit)?;
                    println!("History for {}:", sensor);
                    for n in hist {
                        println!(
                            "  [{}] {} | {}...",
                            n.timestamp_ns,
                            n.value,
                            &n.data_hash[..16]
                        );
                    }
                }
                Commands::Verify { sensor } => match merkle::verify_chain(&sensor, &store) {
                    Ok((count, root)) => {
                        println!("✓ Verified {} nodes, root {}...", count, &root[..16])
                    }
                    Err(e) => {
                        eprintln!("✗ Verification failed: {}", e);
                        std::process::exit(1);
                    }
                },
                Commands::Bench { count } => {
                    use indicatif::ProgressBar;
                    let pb = ProgressBar::new(count as u64);
                    let start = std::time::Instant::now();
                    for i in 0..count {
                        store.put("bench", i as f64, i as u64)?;
                        pb.inc(1);
                    }
                    pb.finish();
                    let dur = start.elapsed();
                    println!(
                        "{} writes in {:.2}s ({:.0} ops/sec)",
                        count,
                        dur.as_secs_f64(),
                        count as f64 / dur.as_secs_f64()
                    );
                }
                Commands::Export { output } => {
                    let file = std::fs::File::create(&output)?;
                    let n = CarExporter::export_all(&store, file)?;
                    println!("✓ Exported {} nodes to {}", n, output);
                }
                Commands::Import { file } => {
                    let file = std::fs::File::open(&file)?;
                    let r = CarExporter::import(&store, file)?;
                    println!("✓ Imported {} (skipped {})", r.imported, r.skipped);
                }
                Commands::Generate {
                    sensor,
                    count,
                    sensor_type,
                } => {
                    println!("Generating {} historical data points for {}", count, sensor);
                    bulk_ops::generate_history(&store, &sensor, count, &sensor_type)?;
                }
                Commands::SimulateLine { line } => {
                    bulk_ops::simulate_line(&store, &line)?;
                }
                Commands::SeedDemo => {
                    let db = init_sqlite_pool(&incident_db).await?;
                    vigil_core::simulation::seed_demo_environment(&db, &store, "./data").await?;
                    println!("✓ Seeded demo data into local store, SQLite, and ./data exports");
                }
                Commands::Detect => {
                    let db = init_sqlite_pool(&incident_db).await?;
                    let summary = run_incident_pipeline(&db, &store).await?;
                    println!(
                        "✓ Incident pipeline complete: {} incidents created from {} events ({} invalid raw events)",
                        summary.created_incident_ids.len(),
                        summary.events_processed,
                        summary.invalid_events
                    );
                }
                _ => {}
            }
        }
    }
    Ok(())
}
