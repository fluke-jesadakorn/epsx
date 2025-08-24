use async_trait::async_trait;
use std::collections::HashMap;

use crate::app::ports::repositories::UsageRepo;
use crate::dom::entities::module::ModuleUsageLog;
use crate::dom::values::UserId;
use crate::dom::error::DomainError;

pub struct DieselUsageRepo {
    // TODO: Add database pool when implementing actual usage tracking
}

impl DieselUsageRepo {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl UsageRepo for DieselUsageRepo {
    async fn log_usage(&self, _usage_log: ModuleUsageLog) -> Result<(), DomainError> {
        // TODO: Implement actual usage logging
        Ok(())
    }

    async fn get_usage_stats(&self, _user_id: &UserId, _module_name: &str) -> Result<HashMap<String, i32>, DomainError> {
        // TODO: Implement actual usage stats retrieval
        Ok(HashMap::new())
    }

    async fn get_current_usage(&self, _user_id: &UserId, _module_name: &str, _quota_type: &str) -> Result<i32, DomainError> {
        // TODO: Implement actual current usage retrieval
        Ok(0)
    }
}