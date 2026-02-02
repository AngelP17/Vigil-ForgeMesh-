pub mod types;
pub mod store;
pub mod merkle;
pub mod simulation;  // NEW
pub mod analytics;   // NEW

pub use types::DataNode;
pub use store::{ForgeStore, StorageError};
pub use merkle::{verify_chain, ChainError};
pub use simulation::IndustrialSimulator;  // NEW
pub use analytics::{SensorStats, OEEMetrics, TrendDirection};  // NEW
