use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GossipMessage {
    RootAnnouncement {
        sensor_id: String,
        root_hash: String,
        node_id: String,
        clock_bytes: Vec<u8>,
    },
    DataRequest {
        sensor_id: String,
        missing_hashes: Vec<String>,
        requester: String,
    },
    DataPayload {
        sensor_id: String,
        nodes: Vec<u8>, // bincode encoded Vec<CrdtNode>
    },
}

pub struct GossipEngine {
    pub node_id: String,
    peers: Arc<RwLock<HashMap<String, PeerState>>>,
}

#[derive(Clone)]
pub struct PeerState {
    pub last_seen: std::time::Instant,
    pub root_hash: String,
}

impl GossipEngine {
    pub fn new(node_id: String) -> Self {
        Self {
            node_id,
            peers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn announce(&self, sensor_id: &str, root_hash: &str, clock_bytes: Vec<u8>) {
        let msg = GossipMessage::RootAnnouncement {
            sensor_id: sensor_id.to_string(),
            root_hash: root_hash.to_string(),
            node_id: self.node_id.clone(),
            clock_bytes,
        };
        debug!("Announcing: {:?}", msg);
        // In real Iroh impl, broadcast here
    }

    pub async fn handle_message(&self, msg: GossipMessage) {
        match msg {
            GossipMessage::RootAnnouncement {
                node_id, root_hash, ..
            } => {
                let mut peers = self.peers.write().await;
                peers.insert(
                    node_id.clone(),
                    PeerState {
                        last_seen: std::time::Instant::now(),
                        root_hash,
                    },
                );
                info!("Peer {} announced new root", node_id);
            }
            _ => {}
        }
    }
}
