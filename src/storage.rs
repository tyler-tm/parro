use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

// Db is just a ref-counted map for now, but I'm setting this up to plug in
//  something better, once I need it.

pub type Db = Arc<Mutex<HashMap<String, String>>>;

pub fn new_db() -> Db {
    Arc::new(Mutex::new(HashMap::new()))
}
