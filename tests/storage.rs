use parro::error::StorageError;
use parro::storage::Store;

#[test]
fn store_get_empty() {
    let store = Store::new(1024);
    assert_eq!(store.get("empty_key"), None);
}

#[test]
fn store_set_and_get() {
    let mut store = Store::new(1024);
    let key = "new_key";
    let value = "new_value";
    store.set(key, value).unwrap();
    assert_eq!(store.get(key), Some(value));
}

#[test]
fn store_update_value() {
    let mut store = Store::new(1024);
    let key = "to_update";
    store.set(key, "old").unwrap();
    let new_value = "updated";
    store.set(key, new_value).unwrap();
    assert_eq!(store.get(key), Some(new_value));
}

#[test]
fn store_set_exceed_limit_new_key() {
    let mut store = Store::new(10);
    let key = "new_key";
    let value = "this_is_a_long_value";
    let result = store.set(key, value);
    assert_eq!(result, Err(StorageError::LimitExceeded));
}

#[test]
fn store_set_exceed_limit_existing_key() {
    let mut store = Store::new(20);
    let key = "to_update";
    let value = "short";
    store.set(key, value).unwrap();
    let new_value = "this_is_a_very_long_new_value";
    let result = store.set(key, new_value);
    assert_eq!(result, Err(StorageError::LimitExceeded));
}

#[test]
fn store_set_empty_value() {
    let mut store = Store::new(1024);
    let key = "new_key";
    let value = "";
    store.set(key, value).unwrap();
    assert_eq!(store.get(key), Some(""));
}

#[test]
fn store_set_empty_key() {
    let mut store = Store::new(1024);
    let key = "";
    let value = "keyless_value";
    store.set(key, value).unwrap();
    assert_eq!(store.get(key), Some(value));
}

#[test]
fn store_delete() {
    let mut store = Store::new(1024);
    let key = "to_delete";
    let value = "some_value";
    store.set(key, value).unwrap();
    assert_eq!(store.get(key), Some(value));
    store.delete(key).unwrap();
    assert_eq!(store.get(key), None);
}

#[test]
fn store_delete_non_existing() {
    let mut store = Store::new(1024);
    let result = store.delete("non_existing");
    assert_eq!(result, Err(StorageError::NotFound));
}
