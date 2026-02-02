use serde::{Serialize, Deserialize};
use sha3::{Sha3_256, Digest};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DataNode {
    pub timestamp_ns: u64,
    pub sensor_id: String,
    pub value: f64,
    pub parent_hash: Option<String>,
    pub data_hash: String,
}

impl DataNode {
    pub fn new(sensor_id: &str, value: f64, timestamp_ns: u64, parent_hash: Option<String>) -> Self {
        let data_hash = Self::calculate_hash(sensor_id, value, timestamp_ns, &parent_hash);
        Self {
            timestamp_ns,
            sensor_id: sensor_id.to_string(),
            value,
            parent_hash,
            data_hash,
        }
    }

    fn calculate_hash(sensor_id: &str, value: f64, timestamp_ns: u64, parent_hash: &Option<String>) -> String {
        let mut hasher = Sha3_256::new();
        hasher.update(sensor_id.as_bytes());
        hasher.update(&timestamp_ns.to_le_bytes());
        hasher.update(&value.to_le_bytes());
        match parent_hash {
            Some(p) => hasher.update(p.as_bytes()),
            None => hasher.update(&[0u8]),
        }
        format!("{:x}", hasher.finalize())
    }

    pub fn verify_integrity(&self) -> bool {
        let computed = Self::calculate_hash(&self.sensor_id, self.value, self.timestamp_ns, &self.parent_hash);
        computed == self.data_hash
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, Box<bincode::ErrorKind>> {
        bincode::serialize(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Box<bincode::ErrorKind>> {
        bincode::deserialize(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_determinism() {
        let n1 = DataNode::new("s1", 10.0, 100, None);
        let n2 = DataNode::new("s1", 10.0, 100, None);
        assert_eq!(n1.data_hash, n2.data_hash);
        assert!(n1.verify_integrity());
    }

    #[test]
    fn test_parent_chain() {
        let g = DataNode::new("s", 1.0, 1, None);
        let c = DataNode::new("s", 2.0, 2, Some(g.data_hash.clone()));
        assert_ne!(g.data_hash, c.data_hash);
        assert_eq!(c.parent_hash, Some(g.data_hash));
        assert!(c.verify_integrity());
    }
}
