use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};
use thiserror::Error;
use vigil_core::{store::ForgeStore, types::DataNode};

#[derive(Error, Debug)]
pub enum CarError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Bincode(String),
    #[error("Storage error: {0}")]
    Storage(#[from] vigil_core::store::StorageError),
    #[error("Invalid magic")]
    InvalidMagic,
}

impl From<Box<bincode::ErrorKind>> for CarError {
    fn from(e: Box<bincode::ErrorKind>) -> Self {
        CarError::Bincode(e.to_string())
    }
}

pub struct ImportReport {
    pub imported: usize,
    pub skipped: usize,
}

pub struct CarExporter;

impl CarExporter {
    pub fn export_all<W: Write>(store: &ForgeStore, mut writer: W) -> Result<usize, CarError> {
        let mut count = 0;
        writer.write_all(b"FORG")?;
        writer.write_u32::<LittleEndian>(1)?;

        for item in store.iter_data() {
            let (hash, node) = item?;
            let data = bincode::serialize(&node)?;

            writer.write_u16::<LittleEndian>(hash.len() as u16)?;
            writer.write_all(hash.as_bytes())?;
            writer.write_u32::<LittleEndian>(data.len() as u32)?;
            writer.write_all(&data)?;
            count += 1;
        }
        Ok(count)
    }

    pub fn import<R: Read>(store: &ForgeStore, mut reader: R) -> Result<ImportReport, CarError> {
        let mut imported = 0;
        let mut skipped = 0;

        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        if &magic != b"FORG" {
            return Err(CarError::InvalidMagic);
        }
        let _ = reader.read_u32::<LittleEndian>()?;

        loop {
            let hash_len = match reader.read_u16::<LittleEndian>() {
                Ok(n) => n,
                Err(_) => break,
            };
            let mut hash_buf = vec![0u8; hash_len as usize];
            reader.read_exact(&mut hash_buf)?;
            let hash = String::from_utf8_lossy(&hash_buf).to_string();

            let data_len = reader.read_u32::<LittleEndian>()?;
            let mut data_buf = vec![0u8; data_len as usize];
            reader.read_exact(&mut data_buf)?;

            let node: DataNode = bincode::deserialize(&data_buf)?;

            match store.get(&hash)? {
                Some(_) => skipped += 1,
                None => {
                    store.insert_raw(&hash, &node)?;
                    if let Some(ref parent) = node.parent_hash {
                        if store.get(parent)?.is_none() && !node.sensor_id.is_empty() {
                            // Orphan handling: in full impl, queue for later
                        }
                    }
                    // Update index if this is latest for sensor
                    let current_latest = store
                        .get_history(&node.sensor_id, 1)
                        .ok()
                        .and_then(|v| v.first().map(|n| n.data_hash.clone()));
                    if current_latest.as_ref() == Some(&hash) || current_latest.is_none() {
                        store.update_index(&node.sensor_id, &hash)?;
                    }
                    imported += 1;
                }
            }
        }

        Ok(ImportReport { imported, skipped })
    }
}
