use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::str::FromStr;
use crate::application::shared::command_bus::{Command, CommandHandler};
use crate::application::shared::ApplicationResult;
use crate::application::shared::error::ApplicationError;
use crate::domain::subscription_management::value_objects::PlanId;
use crate::domain::subscription_management::repository_ports::PlanRepositoryPort;

use crate::application::subscription_management::commands::models::delete_plan::{DeletePlanCommand, DeletePlanResponse};

pub type DeletePlanResult = DeletePlanResponse;

pub struct DeletePlanCommandHandler {
    plan_repository: Arc<dyn PlanRepositoryPort + Send + Sync>,
}

impl DeletePlanCommandHandler {
    pub fn new(plan_repository: Arc<dyn PlanRepositoryPort + Send + Sync>) -> Self {
        Self { plan_repository }
    }
}

#[async_trait]
impl CommandHandler<DeletePlanCommand> for DeletePlanCommandHandler {
    async fn handle(&self, command: DeletePlanCommand) -> ApplicationResult<DeletePlanResult> {
        let plan_id = command.id;

        // 1. Check existence
        let _plan = self.plan_repository.find_by_id(&plan_id)
            .await.map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("Plan", plan_id.to_string()))?;

        // 2. Delete Plan
        self.plan_repository.delete(&plan_id)
            .await.map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        Ok(DeletePlanResult {
            plan_id: plan_id.to_string(),
            deleted: true,
        })
    }
}
