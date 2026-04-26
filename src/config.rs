use crate::static_utils::BYTES_MB_CONVERSION;
use std::env;

const DEFAULT_IP: &str = "127.0.0.1";
const DEFAULT_PORT: &str = "14242";
const DEFAULT_MAX_SIZE_MB: usize = 2048;

pub struct Config {
    pub addr: String,
    pub max_size_bytes: usize,
}

impl Config {
    pub fn from_env() -> Self {
        let port = env::var("PARRO_PORT").unwrap_or_else(|_| DEFAULT_PORT.to_string());
        let max_size_mb: usize = env::var("PARRO_MAX_SIZE_MB")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_MAX_SIZE_MB);
        Config {
            addr: format!("{DEFAULT_IP}:{port}"),
            max_size_bytes: max_size_mb * BYTES_MB_CONVERSION,
        }
    }
}
