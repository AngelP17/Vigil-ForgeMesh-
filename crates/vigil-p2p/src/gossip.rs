use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
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

/// Live view of remote peers seen via gossip (Iroh hook would call `handle_message`).
#[derive(Clone)]
pub struct PeerState {
    pub last_seen: std::time::Instant,
    pub last_seen_wall: DateTime<Utc>,
    pub root_hash: String,
}

#[derive(Clone)]
pub struct GossipEngine {
    pub node_id: String,
    peers: Arc<RwLock<HashMap<String, PeerState>>>,
    last_local_activity: Arc<RwLock<DateTime<Utc>>>,
}

impl GossipEngine {
    pub fn new(node_id: String) -> Self {
        Self {
            node_id,
            peers: Arc::new(RwLock::new(HashMap::new())),
            last_local_activity: Arc::new(RwLock::new(Utc::now())),
        }
    }

    /// Call after local store writes so `/api/status` mesh `last_sync` reflects activity.
    pub async fn touch_local_activity(&self) {
        *self.last_local_activity.write().await = Utc::now();
    }

    /// Distinct mesh participants: this node plus each remote gossip peer.
    pub async fn mesh_node_count(&self) -> i64 {
        let n = self.peers.read().await.len();
        (1 + n) as i64
    }

    pub async fn remote_peer_count(&self) -> usize {
        self.peers.read().await.len()
    }

    /// JSON for `GET /api/status` — mesh block (no hardcoded peers/sync).
    pub async fn mesh_status_json(&self) -> Value {
        let peers_map = self.peers.read().await;
        let local_ts = *self.last_local_activity.read().await;

        let mut last_sync = local_ts;
        let mut peer_rows = Vec::new();
        for (id, st) in peers_map.iter() {
            if st.last_seen_wall > last_sync {
                last_sync = st.last_seen_wall;
            }
            let stale_secs = Utc::now()
                .signed_duration_since(st.last_seen_wall)
                .num_seconds();
            peer_rows.push(json!({
                "node_id": id,
                "root_hash": &st.root_hash[..st.root_hash.len().min(16)],
                "last_seen": st.last_seen_wall.to_rfc3339(),
                "stale_seconds": stale_secs,
            }));
        }

        let peers_connected = peers_map.len();
        let partition_status = if peers_connected == 0 {
            "standalone"
        } else {
            let stale = peers_map
                .values()
                .filter(|p| {
                    Utc::now().signed_duration_since(p.last_seen_wall).num_seconds() > 120
                })
                .count();
            match stale {
                0 => "healthy",
                n if n == peers_connected => "degraded",
                _ => "degraded-but-operational",
            }
        };

        json!({
            "peers_connected": peers_connected,
            "partition_status": partition_status,
            "last_sync": last_sync.to_rfc3339(),
            "peers": peer_rows,
        })
    }

    /// JSON for `GET /api/mesh/topology` — local node + gossip peers; links are star from local.
    pub async fn topology_json(&self) -> Value {
        let peers_map = self.peers.read().await;
        let now = Utc::now();
        let mut nodes = vec![json!({
            "id": self.node_id,
            "role": "local",
            "status": "online",
            "last_seen": "0s",
        })];

        let mut links = Vec::new();

        for (id, st) in peers_map.iter() {
            let ago = now.signed_duration_since(st.last_seen_wall);
            let seen = if ago.num_seconds() < 60 {
                format!("{}s", ago.num_seconds().max(0))
            } else if ago.num_minutes() < 120 {
                format!("{}m", ago.num_minutes())
            } else {
                format!("{}h", ago.num_hours())
            };
            let status = if ago.num_seconds() > 120 {
                "stale"
            } else {
                "online"
            };
            nodes.push(json!({
                "id": id,
                "role": "peer",
                "status": status,
                "last_seen": seen,
                "root_hash_preview": &st.root_hash[..st.root_hash.len().min(12)],
            }));
            links.push(json!({
                "source": self.node_id,
                "target": id,
                "latency_ms": null,
            }));
        }

        json!({
            "source": "gossip",
            "nodes": nodes,
            "links": links,
        })
    }

    pub async fn announce(&self, sensor_id: &str, root_hash: &str, clock_bytes: Vec<u8>) {
        self.touch_local_activity().await;
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
                node_id,
                root_hash,
                ..
            } => {
                if node_id == self.node_id {
                    return;
                }
                let wall = Utc::now();
                let mut peers = self.peers.write().await;
                peers.insert(
                    node_id.clone(),
                    PeerState {
                        last_seen: std::time::Instant::now(),
                        last_seen_wall: wall,
                        root_hash,
                    },
                );
                info!("Peer {} announced new root", node_id);
            }
            _ => {}
        }
    }
}
