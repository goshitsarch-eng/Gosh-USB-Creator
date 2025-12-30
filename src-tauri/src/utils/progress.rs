use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct WriteProgress {
    pub phase: String,
    pub bytes_written: u64,
    pub total_bytes: u64,
    pub speed_bps: u64,
    pub eta_seconds: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChecksumProgress {
    pub bytes_processed: u64,
    pub total_bytes: u64,
}
