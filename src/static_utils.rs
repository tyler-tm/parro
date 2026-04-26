pub const BYTES_MB_CONVERSION: usize = 1024 * 1024;

pub const DEFAULT_IP: &str = "127.0.0.1";
pub const DEFAULT_PORT: &str = "14242";

pub fn default_addr() -> String {
    let port = std::env::var("PARRO_PORT").unwrap_or_else(|_| DEFAULT_PORT.to_string());
    format!("{DEFAULT_IP}:{port}")
}
