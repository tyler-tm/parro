use crate::error::StorageError;
use crate::static_utils::BYTES_MB_CONVERSION;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct Store {
    map: HashMap<String, Vec<u8>>,
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

    pub fn get(&self, key: &str) -> Option<&[u8]> {
        self.map.get(key).map(|v| v.as_slice())
    }

    pub fn set(&mut self, key: &str, value: Vec<u8>) -> Result<(), StorageError> {
        use std::collections::hash_map::Entry;

        let new_value_size = value.len();

        match self.map.entry(key.to_string()) {
            Entry::Occupied(mut entry) => {
                let old_value_size = entry.get().len();
                let new_size_bytes = self.current_size_bytes - old_value_size + new_value_size;

                if new_size_bytes > self.max_size_bytes {
                    println!(
                        "Storage limit exceeded. Max size: {} bytes, attempted to increase existing entry size by {} bytes",
                        self.max_size_bytes,
                        new_value_size - old_value_size
                    );
                    return Err(StorageError::LimitExceeded);
                }
                self.current_size_bytes = new_size_bytes;
                entry.insert(value);
            }
            Entry::Vacant(entry) => {
                let new_key_value_size = entry.key().len() + new_value_size;
                if self.current_size_bytes + new_key_value_size > self.max_size_bytes {
                    println!(
                        "Storage limit exceeded. Max size: {} MB, attempted to add new size of {} bytes",
                        self.max_size_bytes / BYTES_MB_CONVERSION as usize,
                        new_key_value_size
                    );
                    return Err(StorageError::LimitExceeded);
                }
                self.current_size_bytes += new_key_value_size;
                entry.insert(value);
            }
        }
        Ok(())
    }

    pub fn delete(&mut self, key: &str) -> Result<(), StorageError> {
        if let Some(value) = self.map.remove(key) {
            self.current_size_bytes -= key.len() + value.len();
            Ok(())
        } else {
            Err(StorageError::NotFound)
        }
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

    let max_size_bytes = max_size_mb * BYTES_MB_CONVERSION as usize;
    Arc::new(RwLock::new(Store::new(max_size_bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_get_empty() {
        let store = Store::new(1);
        assert_eq!(store.get("empty_key"), None);
    }

    #[test]
    fn store_set_and_get() {
        let mut store = Store::new(32);
        store.set("new_key", b"new_value".to_vec()).unwrap();
        assert_eq!(store.get("new_key"), Some(b"new_value".as_slice()));
    }

    #[test]
    fn store_update_value() {
        let mut store = Store::new(32);
        store.set("to_update", b"old".to_vec()).unwrap();
        store.set("to_update", b"updated".to_vec()).unwrap();
        assert_eq!(store.get("to_update"), Some(b"updated".as_slice()));
    }

    #[test]
    fn store_set_exceed_limit_new_key() {
        let mut store = Store::new(16);
        let result = store.set("new_key", b"this_is_a_long_value".to_vec());
        assert_eq!(result, Err(StorageError::LimitExceeded));
    }

    #[test]
    fn store_set_exceed_limit_existing_key() {
        let mut store = Store::new(16);
        store.set("to_update", b"short".to_vec()).unwrap();
        let result = store.set("to_update", b"this_is_a_very_long_new_value".to_vec());
        assert_eq!(result, Err(StorageError::LimitExceeded));
    }

    #[test]
    fn store_set_empty_value() {
        let mut store = Store::new(8);
        store.set("new_key", vec![]).unwrap();
        assert_eq!(store.get("new_key"), Some(b"".as_slice()));
    }

    #[test]
    fn store_set_empty_key() {
        let mut store = Store::new(8);
        store.set("", b"keyless".to_vec()).unwrap();
        assert_eq!(store.get(""), Some(b"keyless".as_slice()));
    }

    #[test]
    fn store_delete() {
        let mut store = Store::new(32);
        store.set("to_delete", b"some_value".to_vec()).unwrap();
        assert_eq!(store.get("to_delete"), Some(b"some_value".as_slice()));
        store.delete("to_delete").unwrap();
        assert_eq!(store.get("to_delete"), None);
    }

    #[test]
    fn store_delete_non_existing() {
        let mut store = Store::new(1);
        let result = store.delete("non_existing");
        assert_eq!(result, Err(StorageError::NotFound));
    }
}
