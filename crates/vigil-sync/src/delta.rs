use vigil_core::store::ForgeStore;

pub struct DeltaCalculator;

impl DeltaCalculator {
    pub fn find_missing_hashes(
        local: &ForgeStore,
        remote_hashes: &[String],
    ) -> Result<Vec<String>, vigil_core::store::StorageError> {
        let mut missing = Vec::new();
        for hash in remote_hashes {
            if local.get(hash)?.is_none() {
                missing.push(hash.clone());
            }
        }
        Ok(missing)
    }
}
