use crate::prelude::*;
use crate::application::shared::Command;
use std::collections::HashMap;

/// Command to update resource quotas for a plan
#[derive(Debug, Clone)]
pub struct UpdateResourceQuotaCommand {
    pub plan_id: i32,
    pub resource_limits: HashMap<String, i64>,
}

impl Command for UpdateResourceQuotaCommand {
    type Response = UpdateResourceQuotaResponse;
}

/// Response after updating resource quotas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResourceQuotaResponse {
    pub plan_id: i32,
    pub updated_resources: Vec<String>,
    pub updated_at: DateTime<Utc>,
}
