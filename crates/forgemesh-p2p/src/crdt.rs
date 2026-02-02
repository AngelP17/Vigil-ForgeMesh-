use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::collections::HashSet;
use forgemesh_core::types::DataNode;

pub type NodeId = String;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VectorClock {
    pub clock: HashMap<NodeId, u64>,
}

impl VectorClock {
    pub fn new() -> Self {
        Self { clock: HashMap::new() }
    }

    pub fn increment(&mut self, node: &NodeId) {
        *self.clock.entry(node.clone()).or_insert(0) += 1;
    }

    pub fn merge(&mut self, other: &VectorClock) {
        for (k, v) in &other.clock {
            let entry = self.clock.entry(k.clone()).or_insert(0);
            if *entry < *v {
                *entry = *v;
            }
        }
    }

    pub fn compare(&self, other: &VectorClock) -> Option<std::cmp::Ordering> {
        let mut dom = false;
        let mut sub = false;
        let keys: HashSet<_> = self.clock.keys().chain(other.clock.keys()).cloned().collect();
        
        for k in keys {
            let s = self.clock.get(&k).copied().unwrap_or(0);
            let o = other.clock.get(&k).copied().unwrap_or(0);
            if s > o { dom = true; }
            if s < o { sub = true; }
        }
        
        match (dom, sub) {
            (true, false) => Some(std::cmp::Ordering::Greater),
            (false, true) => Some(std::cmp::Ordering::Less),
            (false, false) => Some(std::cmp::Ordering::Equal),
            (true, true) => None, // Concurrent
        }
    }
}

impl Default for VectorClock {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrdtNode {
    pub node: DataNode,
    pub clock: VectorClock,
    pub node_id: NodeId,
}

impl CrdtNode {
    pub fn new(node: DataNode, node_id: NodeId, clock: VectorClock) -> Self {
        Self { node, node_id, clock }
    }
}
