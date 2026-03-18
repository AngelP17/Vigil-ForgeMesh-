use crate::types::DataNode;
use sled::{Batch, Db};
use std::path::Path;
use thiserror::Error;
use tracing::{debug, info};

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Sled(#[from] sled::Error),
    #[error("Serialization error: {0}")]
    Bincode(#[from] Box<bincode::ErrorKind>),
    #[error("Corruption detected at hash {0}")]
    Corruption(String),
}

pub struct ForgeStore {
    db: Db,
}

impl ForgeStore {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let db = sled::open(&path)?;
        info!("ForgeStore opened at {:?}", path.as_ref());
        Ok(Self { db })
    }

    pub fn put(
        &self,
        sensor_id: &str,
        value: f64,
        timestamp_ns: u64,
    ) -> Result<String, StorageError> {
        let parent = self.get_latest_hash(sensor_id)?;
        let node = DataNode::new(sensor_id, value, timestamp_ns, parent);
        let hash = node.data_hash.clone();
        let bytes = bincode::serialize(&node)?;

        let mut batch = Batch::default();
        batch.insert(format!("data:{hash}").as_bytes(), bytes);
        batch.insert(format!("idx:{sensor_id}").as_bytes(), hash.as_bytes());

        self.db.apply_batch(batch)?;
        debug!("Stored {} for {}", &hash[..8], sensor_id);
        Ok(hash)
    }

    pub fn insert_raw(&self, hash: &str, node: &DataNode) -> Result<(), StorageError> {
        if !node.verify_integrity() {
            return Err(StorageError::Corruption(hash.to_string()));
        }
        let bytes = bincode::serialize(node)?;
        self.db.insert(format!("data:{hash}").as_bytes(), bytes)?;
        Ok(())
    }

    pub fn get(&self, hash: &str) -> Result<Option<DataNode>, StorageError> {
        let key = format!("data:{hash}");
        match self.db.get(key)? {
            Some(ivec) => {
                let node: DataNode = bincode::deserialize(&ivec)?;
                if !node.verify_integrity() {
                    return Err(StorageError::Corruption(hash.to_string()));
                }
                Ok(Some(node))
            }
            None => Ok(None),
        }
    }

    pub fn get_latest_hash(&self, sensor_id: &str) -> Result<Option<String>, StorageError> {
        let key = format!("idx:{sensor_id}");
        Ok(self
            .db
            .get(key)?
            .map(|v| String::from_utf8_lossy(&v).to_string()))
    }

    pub fn get_latest(&self, sensor_id: &str) -> Result<Option<DataNode>, StorageError> {
        match self.get_latest_hash(sensor_id)? {
            Some(h) => self.get(&h),
            None => Ok(None),
        }
    }

    pub fn get_history(
        &self,
        sensor_id: &str,
        limit: usize,
    ) -> Result<Vec<DataNode>, StorageError> {
        let mut results = Vec::with_capacity(limit);
        let mut current = self.get_latest_hash(sensor_id)?;

        while let Some(hash) = current {
            if results.len() >= limit {
                break;
            }
            match self.get(&hash)? {
                Some(node) => {
                    current = node.parent_hash.clone();
                    results.push(node);
                }
                None => break,
            }
        }
        Ok(results)
    }

    pub fn iter_data(&self) -> impl Iterator<Item = Result<(String, DataNode), StorageError>> + '_ {
        self.db.scan_prefix("data:").map(|item| {
            let (k, v) = item?;
            let hash = String::from_utf8_lossy(&k[5..]).to_string();
            let node: DataNode = bincode::deserialize(&v)?;
            Ok((hash, node))
        })
    }

    pub fn get_db(&self) -> &Db {
        &self.db
    }

    pub fn update_index(&self, sensor_id: &str, hash: &str) -> Result<(), StorageError> {
        self.db
            .insert(format!("idx:{sensor_id}").as_bytes(), hash.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_put_get() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let store = ForgeStore::new(dir.path())?;
        let h = store.put("s1", 42.0, 100)?;
        let node = store.get(&h)?.unwrap();
        assert_eq!(node.value, 42.0);
        Ok(())
    }
}
