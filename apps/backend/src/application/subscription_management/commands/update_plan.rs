use chrono::Utc;
use async_trait::async_trait;
use std::sync::Arc;
use crate::application::shared::command_bus::CommandHandler;
use crate::application::shared::ApplicationResult;
use crate::application::shared::error::ApplicationError;

use crate::domain::subscription_management::aggregates::{UpdatePlanParams};
use crate::domain::subscription_management::value_objects::Price;
use crate::domain::subscription_management::repository_ports::PlanRepositoryPort;

use crate::application::subscription_management::commands::models::update_plan::{UpdatePlanCommand, UpdatePlanResponse};

pub type UpdatePlanResult = UpdatePlanResponse;

pub struct UpdatePlanCommandHandler {
    plan_repository: Arc<dyn PlanRepositoryPort + Send + Sync>,
}

impl UpdatePlanCommandHandler {
    pub fn new(plan_repository: Arc<dyn PlanRepositoryPort + Send + Sync>) -> Self {
        Self { plan_repository }
    }
}

#[async_trait]
impl CommandHandler<UpdatePlanCommand> for UpdatePlanCommandHandler {
    async fn handle(&self, command: UpdatePlanCommand) -> ApplicationResult<UpdatePlanResult> {
        let plan_id = command.id;

        // 1. Load Plan
        let mut plan = self.plan_repository.find_by_id(&plan_id)
            .await.map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("Plan", plan_id.to_string()))?;

        // 2. Prepare Update Params
        let price = if let (Some(amount), Some(currency)) = (command.price, command.currency) {
            Some(Price::new(amount, currency).map_err(|e| ApplicationError::validation("price", e.to_string()))?)
        } else {
            None
        };

        let params = UpdatePlanParams {
            name: command.name,
            description: command.description,
            price,
            billing_cycle: command.billing_cycle,
            features: command.features,
            target_audience: command.target_audience,
            permissions: command.permissions,
            is_active: command.is_active,
            is_promoted: command.is_promoted,
            display_order: command.display_order,
            metadata: command.metadata,
        };

        // 3. Update Plan (Domain Logic)
        plan.update(params).map_err(|e| ApplicationError::business_rule(e.to_string()))?;

        // 4. Save Plan
        self.plan_repository.save(&plan).await.map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        Ok(UpdatePlanResult {
            plan_id: plan_id.to_string(),
            updated_at: Utc::now(),
        })
    }
}
