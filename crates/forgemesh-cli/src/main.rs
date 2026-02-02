use clap::{Parser, Subcommand};
use forgemesh_core::store::ForgeStore;
use forgemesh_core::merkle;
use forgemesh_sync::car::CarExporter;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::Result;
use tracing::info;

mod bulk_ops;

#[derive(Parser)]
#[command(name="forgemesh")]
#[command(about="Distributed Industrial Historian - Zero Cost, Maximum Resilience")]
struct Cli {
    #[arg(long, default_value="./forge_data")]
    db_path: String,
    #[arg(long, default_value="node-local")]
    node_id: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Write{#[arg(short,long)]sensor:String,#[arg(short,long)]value:f64},
    Log{#[arg(short,long)]sensor:String,#[arg(short='n',long,default_value_t=10)]limit:usize},
    Verify{#[arg(short,long)]sensor:String},
    Bench{#[arg(long,default_value_t=10000)]count:usize},
    Export{#[arg(short,long)]output:String},
    Import{#[arg(short,long)]file:String},
    Daemon{#[arg(long,default_value_t=8080)]port:u16},
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
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli=Cli::parse();
    
    match cli.command {
        Commands::Daemon{port}=>{
            let store=Arc::new(Mutex::new(ForgeStore::new(&cli.db_path)?));
            let (tx,_rx)=tokio::sync::broadcast::channel(256);
            let state=Arc::new(forgemesh_web::AppState{store:store.clone(),node_id:cli.node_id.clone(),tx});
            let app=forgemesh_web::create_router(state);
            let addr=format!("0.0.0.0:{}",port);
            info!("Daemon starting on http://{}",addr);
            println!("ForgeMesh UI: http://{}:{}", "localhost", port);
            axum::serve(tokio::net::TcpListener::bind(&addr).await?,app).await?;
        },
        cmd=>{
            let store=ForgeStore::new(&cli.db_path)?;
            match cmd {
                Commands::Write{sensor,value}=>{
                    let ts=SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos() as u64;
                    let hash=store.put(&sensor,value,ts)?;
                    println!("✓ Written {} = {} (hash {}...)", sensor, value, &hash[..16]);
                },
                Commands::Log{sensor,limit}=>{
                    let hist=store.get_history(&sensor,limit)?;
                    println!("History for {}:",sensor);
                    for n in hist {
                        println!("  [{}] {} | {}...", n.timestamp_ns, n.value, &n.data_hash[..16]);
                    }
                },
                Commands::Verify{sensor}=>{
                    match merkle::verify_chain(&sensor,&store) {
                        Ok((count,root))=>println!("✓ Verified {} nodes, root {}...", count, &root[..16]),
                        Err(e)=>{eprintln!("✗ Verification failed: {}",e);std::process::exit(1);}
                    }
                },
                Commands::Bench{count}=>{
                    use indicatif::ProgressBar;
                    let pb=ProgressBar::new(count as u64);
                    let start=std::time::Instant::now();
                    for i in 0..count { store.put("bench",i as f64,i as u64)?; pb.inc(1); }
                    pb.finish();
                    let dur=start.elapsed();
                    println!("{} writes in {:.2}s ({:.0} ops/sec)",count,dur.as_secs_f64(),count as f64/dur.as_secs_f64());
                },
                Commands::Export{output}=>{
                    let file=std::fs::File::create(&output)?;
                    let n=CarExporter::export_all(&store,file)?;
                    println!("✓ Exported {} nodes to {}",n,output);
                },
                Commands::Import{file}=>{
                    let file=std::fs::File::open(&file)?;
                    let r=CarExporter::import(&store,file)?;
                    println!("✓ Imported {} (skipped {})",r.imported,r.skipped);
                },
                Commands::Generate { sensor, count, sensor_type } => {
                    println!("Generating {} historical data points for {}", count, sensor);
                    bulk_ops::generate_history(&store, &sensor, count, &sensor_type)?;
                }
                Commands::SimulateLine { line } => {
                    bulk_ops::simulate_line(&store, &line)?;
                }
                _=>{}
            }
        }
    }
    Ok(())
}
