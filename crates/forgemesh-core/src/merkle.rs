use crate::store::{ForgeStore, StorageError};
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
    let head = store.get_latest(sensor_id)?
        .ok_or_else(|| ChainError::Empty(sensor_id.to_string()))?;
    
    let mut current = head;
    let mut count = 0;

    loop {
        if !current.verify_integrity() {
            return Err(ChainError::InvalidHash(current.data_hash.clone()));
        }
        count += 1;

        match &current.parent_hash {
            Some(parent_hash) => {
                match store.get(parent_hash)? {
                    Some(parent) => current = parent,
                    None => return Err(ChainError::BrokenLink(current.data_hash.clone(), parent_hash.clone())),
                }
            }
            None => return Ok((count, current.data_hash.clone())),
        }
    }
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
