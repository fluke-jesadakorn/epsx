// Sync EPS Data Command
// Trigger manual TradingView data synchronization

use serde::{Deserialize, Serialize};
use crate::application::shared::Command;

#[derive(Debug, Clone)]
pub struct SyncEPSDataCommand {
    pub country: Option<String>,
    pub sector: Option<String>,
    pub force_full_sync: bool,
}

impl Command for SyncEPSDataCommand {
    type Response = SyncEPSDataResponse;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncEPSDataResponse {
    pub success: bool,
    pub synced_symbols: i32,
    pub updated_records: i32,
    pub new_records: i32,
    pub duration_ms: u64,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
