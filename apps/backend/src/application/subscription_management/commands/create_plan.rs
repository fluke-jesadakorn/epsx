use async_trait::async_trait;
use std::sync::Arc;
use crate::application::shared::command_bus::CommandHandler;

use crate::application::shared::ApplicationResult;
use crate::application::shared::error::ApplicationError;

use crate::domain::subscription_management::domain_services::plan_factory::PlanFactory;
use crate::domain::subscription_management::repository_ports::PlanRepositoryPort;

use crate::application::subscription_management::commands::models::create_plan::{CreatePlanCommand, CreatePlanResponse};

pub type CreatePlanResult = CreatePlanResponse;

pub struct CreatePlanCommandHandler {
    plan_repository: Arc<dyn PlanRepositoryPort + Send + Sync>,
}

impl CreatePlanCommandHandler {
    pub fn new(plan_repository: Arc<dyn PlanRepositoryPort + Send + Sync>) -> Self {
        Self { plan_repository }
    }
}

#[async_trait]
impl CommandHandler<CreatePlanCommand> for CreatePlanCommandHandler {
    async fn handle(&self, command: CreatePlanCommand) -> ApplicationResult<CreatePlanResult> {
        let permission_group = PlanFactory::derive_group_from_permissions(&command.permissions);

        let plan = PlanFactory::create_plan(
            command.name.clone(),
            Some(command.description),
            permission_group,
            command.permissions,
            command.price_amount,
            command.currency,
            command.billing_cycle,
        ).map_err(ApplicationError::business_rule)?;

        self.plan_repository.save(&plan).await.map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        Ok(CreatePlanResult {
            plan_id: plan.id().to_string(),
            name: plan.name().to_string(),
        })
    }
}


