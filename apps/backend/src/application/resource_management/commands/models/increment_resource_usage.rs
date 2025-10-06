use crate::prelude::*;
use crate::application::shared::Command;

/// Command to increment resource usage for a wallet
#[derive(Debug, Clone)]
pub struct IncrementResourceUsageCommand {
    pub wallet_address: String,
    pub resource_type: String,
    pub amount: i64,
}

impl Command for IncrementResourceUsageCommand {
    type Response = IncrementResourceUsageResponse;
}

/// Response after incrementing resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementResourceUsageResponse {
    pub wallet_address: String,
    pub resource_type: String,
    pub current_usage: i64,
    pub quota_limit: i64,
    pub usage_percentage: f64,
    pub limit_exceeded: bool,
}
