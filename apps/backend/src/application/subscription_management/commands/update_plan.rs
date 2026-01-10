use chrono::Utc;
use async_trait::async_trait;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::str::FromStr;
use crate::application::shared::command_bus::{Command, CommandHandler};
use crate::application::shared::ApplicationResult;
use crate::application::shared::error::ApplicationError;

use crate::domain::subscription_management::aggregates::{UpdatePlanParams};
use crate::domain::subscription_management::value_objects::{PlanId, Price, BillingCycle};
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
        let mut params = UpdatePlanParams::default();
        params.name = command.name;
        params.description = command.description;
        
        if let (Some(amount), Some(currency)) = (command.price, command.currency) {
            params.price = Some(Price::new(amount, currency).map_err(|e| ApplicationError::validation("price", e.to_string()))?);
        }
        
        params.billing_cycle = command.billing_cycle;
        params.is_active = command.is_active;
        params.is_promoted = command.is_promoted;
        params.display_order = command.display_order;
        params.permissions = command.permissions;
        params.metadata = command.metadata;

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
