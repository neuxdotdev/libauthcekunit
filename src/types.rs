use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsrfToken {
    pub value: String,
    pub source_url: String,
    pub extracted_at: std::time::SystemTime,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchOptions {
    pub timeout_secs: u64,
    pub max_retries: u32,
    pub retry_initial_interval_ms: u64,
    pub follow_redirects: bool,
    pub max_redirects: u32,
}
