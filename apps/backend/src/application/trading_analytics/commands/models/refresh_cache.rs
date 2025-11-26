// Refresh Cache Command
// Force refresh analytics cache

use serde::{Deserialize, Serialize};
use crate::application::shared::Command;

#[derive(Debug, Clone)]
pub struct RefreshCacheCommand {
    pub cache_key_pattern: Option<String>,
    pub force_full_refresh: bool,
}

impl Command for RefreshCacheCommand {
    type Response = RefreshCacheResponse;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshCacheResponse {
    pub success: bool,
    pub refreshed_entries: usize,
    pub duration_ms: u64,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
