use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::resource_management::commands::{
    IncrementResourceUsageCommand, IncrementResourceUsageResponse
};
use crate::domain::resource_management::repository_ports::UserResourceUsageRepository;

/// Handler for incrementing resource usage
pub struct IncrementResourceUsageCommandHandler<R: UserResourceUsageRepository> {
    usage_repository: Arc<R>,
}

impl<R: UserResourceUsageRepository> IncrementResourceUsageCommandHandler<R> {
    pub fn new(usage_repository: Arc<R>) -> Self {
        Self { usage_repository }
    }
}

#[async_trait]
impl<R: UserResourceUsageRepository + Send + Sync> CommandHandler<IncrementResourceUsageCommand>
    for IncrementResourceUsageCommandHandler<R>
where
    R::Error: std::fmt::Display,
{
    async fn handle(&self, command: IncrementResourceUsageCommand) -> ApplicationResult<IncrementResourceUsageResponse> {
        // 1. Retrieve or create usage aggregate
        let mut usage = self.usage_repository
            .get_user_usage(&command.wallet_address, "default")
            .await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("usage", command.wallet_address.clone()))?;

        // 2. Increment usage
        usage.increment_usage(command.resource_type.clone(), command.amount)
            .map_err(|e| ApplicationError::business_rule(e.to_string()))?;

        let current_usage = *usage.current_usage().get(&command.resource_type).unwrap_or(&0);
        let quota_limit = *usage.quota_limits().get(&command.resource_type).unwrap_or(&0);
        let usage_percentage = usage.get_usage_percentage(&command.resource_type);
        let limit_exceeded = usage.is_limit_exceeded(&command.resource_type);

        // 3. Save updated usage
        self.usage_repository
            .update_user_usage(&usage)
            .await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 4. Return response
        Ok(IncrementResourceUsageResponse {
            wallet_address: command.wallet_address,
            resource_type: command.resource_type,
            current_usage,
            quota_limit,
            usage_percentage,
            limit_exceeded,
        })
    }
}
