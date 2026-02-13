use crate::error::StorageError;
use crate::static_utils::{bytes_to_mb, mb_to_bytes};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct Store {
    map: HashMap<String, String>,
    current_size_bytes: usize,
    max_size_bytes: usize,
}

impl Store {
    pub fn new(max_size_bytes: usize) -> Self {
        Store {
            map: HashMap::new(),
            current_size_bytes: 0,
            max_size_bytes,
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.map.get(key).map(|s| s.as_str())
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<(), StorageError> {
        let new_value_size = value.len();

        if let Some(old_value) = self.map.get(key) {
            let new_size_bytes = self.current_size_bytes - old_value.len() + new_value_size;

            if new_size_bytes > self.max_size_bytes {
                println!(
                    "Storage limit exceeded. Max size: {} bytes, attempted to increase existing entry size by {} bytes",
                    self.max_size_bytes,
                    new_value_size - old_value.len()
                );
                return Err(StorageError::LimitExceeded);
            }
            self.current_size_bytes = new_size_bytes;
        } else {
            let new_key_value_size = key.len() + new_value_size;
            if self.current_size_bytes + new_key_value_size > self.max_size_bytes {
                println!(
                    "Storage limit exceeded. Max size: {} MB, attempted to add new size of {} bytes",
                    bytes_to_mb(self.max_size_bytes),
                    new_key_value_size
                );
                return Err(StorageError::LimitExceeded);
            }
            self.current_size_bytes += new_key_value_size;
        }

        self.map.insert(key.to_string(), value.to_string());
        Ok(())
    }
}

pub type Db = Arc<RwLock<Store>>;

pub fn new_db() -> Db {
    const DEFAULT_MAX_SIZE_MB: usize = 2048;
    let max_size_mb: usize = env::var("PARRO_MAX_SIZE_MB")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_MAX_SIZE_MB);
    println!("Max size: {} MB", max_size_mb);

    let max_size_bytes = mb_to_bytes(max_size_mb);
    Arc::new(RwLock::new(Store::new(max_size_bytes)))
}
