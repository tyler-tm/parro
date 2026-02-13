const BYTES_MB_CONVERSION: usize = 1024 * 1024;

pub const fn mb_to_bytes(mb: usize) -> usize {
    mb * BYTES_MB_CONVERSION
}

pub const fn bytes_to_mb(bytes: usize) -> usize {
    bytes / BYTES_MB_CONVERSION
}
