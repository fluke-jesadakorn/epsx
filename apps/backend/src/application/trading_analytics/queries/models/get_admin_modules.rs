// Get Admin Modules Query
// Module usage statistics and status for admin monitoring

use serde::{Deserialize, Serialize};
use crate::application::shared::Query;

#[derive(Debug, Clone)]
pub struct GetAdminModulesQuery {
    pub include_inactive: Option<bool>,
}

impl Query for GetAdminModulesQuery {
    type Response = GetAdminModulesResponse;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetAdminModulesResponse {
    pub success: bool,
    pub modules: Vec<ModuleStatus>,
    pub total_active: i32,
    pub total_inactive: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleStatus {
    pub module_name: String,
    pub module_type: String,
    pub status: String,
    pub total_requests: i64,
    pub last_accessed: Option<chrono::DateTime<chrono::Utc>>,
    pub average_response_time_ms: f64,
    pub error_rate: f64,
}
