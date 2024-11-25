use std::{fs::OpenOptions, io::{self, Read, Write}};
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use bincode;

#[derive(Serialize, Deserialize, Debug)]
pub struct Record {
    pub key: String,
    pub value: Vec<u8>,
    pub checksum: Vec<u8>,
}

pub struct Database {
    file_path: String,
}

impl Database {
    pub fn new(file_path: String) -> Self {
        Database { file_path }
    }

    pub fn insert(&self, key: &str, value: &[u8]) -> io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(&self.file_path)?;

        let checksum = self.generate_checksum(value);
        let record = Record {
            key: key.to_string(),
            value: value.to_vec(),
            checksum,
        };

        let encoded: Vec<u8> = bincode::serialize(&record).unwrap();
        file.write_all(&encoded)?;

        log::info!("Inserted record with key: {}", key);
        Ok(())
    }

    pub fn get(&self, key: &str) -> io::Result<Option<Vec<u8>>> {
        let mut file = OpenOptions::new().read(true).open(&self.file_path)?;

        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let mut cursor = io::Cursor::new(buffer);
        while let Ok(record) = bincode::deserialize_from::<_, Record>(&mut cursor) {
            if record.key == key {
                if self.validate_checksum(&record.value, &record.checksum) {
                    log::info!("Fetched record with key: {}", key);
                    return Ok(Some(record.value));
                } else {
                    log::warn!("Checksum mismatch for key: {}", key);
                }
            }
        }

        Ok(None)
    }

    fn generate_checksum(&self, data: &[u8]) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().to_vec()
    }

    fn validate_checksum(&self, data: &[u8], checksum: &[u8]) -> bool {
        self.generate_checksum(data) == checksum
    }
}
