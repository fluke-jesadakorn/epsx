use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::resource_management::commands::{
    UpdateResourceQuotaCommand, UpdateResourceQuotaResponse
};
use crate::domain::resource_management::repository_ports::PlanResourceConfigRepository;

/// Handler for updating resource quotas
pub struct UpdateResourceQuotaCommandHandler<R: PlanResourceConfigRepository> {
    plan_repository: Arc<R>,
}

impl<R: PlanResourceConfigRepository> UpdateResourceQuotaCommandHandler<R> {
    pub fn new(plan_repository: Arc<R>) -> Self {
        Self { plan_repository }
    }
}

#[async_trait]
impl<R: PlanResourceConfigRepository + Send + Sync> CommandHandler<UpdateResourceQuotaCommand>
    for UpdateResourceQuotaCommandHandler<R>
where
    R::Error: std::fmt::Display,
{
    async fn handle(&self, command: UpdateResourceQuotaCommand) -> ApplicationResult<UpdateResourceQuotaResponse> {
        // 1. Retrieve plan config
        let mut config = self.plan_repository
            .get_plan_config(command.plan_id)
            .await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("plan_config", command.plan_id.to_string()))?;

        // 2. Update resource limits
        let mut updated_resources = Vec::new();
        for (resource_type, limit) in command.resource_limits {
            config.set_resource_limit(resource_type.clone(), limit)
                .map_err(|e| ApplicationError::business_rule(e.to_string()))?;
            updated_resources.push(resource_type);
        }

        let updated_at = Utc::now();

        // 3. Save updated config
        self.plan_repository
            .update_plan_config(&config)
            .await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 4. Return response
        Ok(UpdateResourceQuotaResponse {
            plan_id: command.plan_id,
            updated_resources,
            updated_at,
        })
    }
}
