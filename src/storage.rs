use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// Db is just a map behind a RwLock for now, but I'm setting this up as a way
//  to plug in something better, once I need it.

pub type Db = Arc<RwLock<HashMap<String, String>>>;

pub fn new_db() -> Db {
    Arc::new(RwLock::new(HashMap::new()))
}
