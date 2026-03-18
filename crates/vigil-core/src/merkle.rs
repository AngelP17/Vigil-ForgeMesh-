use crate::store::{ForgeStore, StorageError};
use sha3::{Digest, Sha3_256};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChainError {
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error("Broken link: {0} -> {1}")]
    BrokenLink(String, String),
    #[error("Invalid hash: {0}")]
    InvalidHash(String),
    #[error("Empty chain for {0}")]
    Empty(String),
}

pub fn verify_chain(sensor_id: &str, store: &ForgeStore) -> Result<(usize, String), ChainError> {
    let head = store
        .get_latest(sensor_id)?
        .ok_or_else(|| ChainError::Empty(sensor_id.to_string()))?;

    let mut current = head;
    let mut count = 0;

    loop {
        if !current.verify_integrity() {
            return Err(ChainError::InvalidHash(current.data_hash.clone()));
        }
        count += 1;

        match &current.parent_hash {
            Some(parent_hash) => match store.get(parent_hash)? {
                Some(parent) => current = parent,
                None => {
                    return Err(ChainError::BrokenLink(
                        current.data_hash.clone(),
                        parent_hash.clone(),
                    ))
                }
            },
            None => return Ok((count, current.data_hash.clone())),
        }
    }
}

pub fn compute_merkle_root(leaves: &[String]) -> String {
    if leaves.is_empty() {
        return hash_leaf("empty");
    }

    let mut level: Vec<String> = leaves.iter().map(|leaf| hash_leaf(leaf)).collect();
    while level.len() > 1 {
        let mut next = Vec::with_capacity(level.len().div_ceil(2));
        for chunk in level.chunks(2) {
            let left = &chunk[0];
            let right = chunk.get(1).unwrap_or(left);
            next.push(hash_leaf(&format!("{left}{right}")));
        }
        level = next;
    }

    level[0].clone()
}

pub fn build_merkle_proof(leaves: &[String]) -> Vec<String> {
    if leaves.is_empty() {
        return Vec::new();
    }

    let mut proof = Vec::new();
    let mut level: Vec<String> = leaves.iter().map(|leaf| hash_leaf(leaf)).collect();
    while level.len() > 1 {
        proof.extend(level.clone());
        let mut next = Vec::with_capacity(level.len().div_ceil(2));
        for chunk in level.chunks(2) {
            let left = &chunk[0];
            let right = chunk.get(1).unwrap_or(left);
            next.push(hash_leaf(&format!("{left}{right}")));
        }
        level = next;
    }
    proof.push(level[0].clone());
    proof
}

fn hash_leaf(value: &str) -> String {
    let mut hasher = Sha3_256::new();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::ForgeStore;
    use tempfile::tempdir;

    #[test]
    fn test_verify_chain() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let store = ForgeStore::new(dir.path())?;
        store.put("s", 1.0, 1)?;
        store.put("s", 2.0, 2)?;
        let (count, _) = verify_chain("s", &store)?;
        assert_eq!(count, 2);
        Ok(())
    }
}
