use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::subscription_management::commands::{
    CreatePlanCommand, CreatePlanResponse
};
use crate::domain::subscription_management::{
    Plan, PlanId, PlanRepositoryPort, Price, BillingCycle, PlanFeatures, CreatePlanParams
};
use rust_decimal::Decimal;
use std::str::FromStr;
use crate::domain::permission_management::GroupId;
use crate::domain::shared_kernel::DomainEventBus;

/// Command handler for creating plans
pub struct CreatePlanCommandHandler {
    plan_repository: Arc<dyn PlanRepositoryPort>,
    event_bus: Arc<dyn DomainEventBus>,
}

impl CreatePlanCommandHandler {
    pub fn new(
        plan_repository: Arc<dyn PlanRepositoryPort>,
        event_bus: Arc<dyn DomainEventBus>,
    ) -> Self {
        Self {
            plan_repository,
            event_bus,
        }
    }
}

#[async_trait]
impl CommandHandler<CreatePlanCommand> for CreatePlanCommandHandler {
    async fn handle(&self, command: CreatePlanCommand) -> ApplicationResult<CreatePlanResponse> {
        // 1. Parse permission group ID
        let permission_group_id = GroupId::from_str(&command.permission_group_id)
            .map_err(|e| ApplicationError::validation("permission_group_id", e.to_string()))?;

        // 2. Create price value object - convert f64 to Decimal
        let price_decimal = Decimal::from_str(&command.price.to_string())
            .map_err(|e| ApplicationError::validation("price", format!("Invalid price format: {}", e)))?;
        let price = Price::new(price_decimal, command.currency.clone())
            .map_err(|e| ApplicationError::validation("price", e.to_string()))?;

        // 3. Parse billing cycle
        let billing_cycle = BillingCycle::from_str(&command.billing_cycle)
            .map_err(|e| ApplicationError::validation("billing_cycle", e.to_string()))?;

        // 4. Create features
        let features = PlanFeatures::new(
            command.api_calls_limit,
            command.rankings_limit,
            command.analytics_enabled,
            command.premium_support,
        );

        // 5. Create plan ID
        let plan_id = PlanId::new();

        // 6. Create plan aggregate
        let plan = Plan::create(plan_id.clone(), CreatePlanParams {
            name: command.name.clone(),
            description: command.description,
            permission_group_id,
            price,
            billing_cycle,
            features,
            target_audience: command.target_audience,
            is_active: command.is_active,
            is_promoted: command.is_promoted,
            display_order: command.display_order,
            metadata: command.metadata,
        }).map_err(|e| ApplicationError::business_logic(e.to_string()))?;

        // 7. Save plan
        self.plan_repository.save(&plan).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 8. Publish domain events
        for event in plan.uncommitted_events() {
            self.event_bus.publish(&**event);
        }

        // 9. Return response
        Ok(CreatePlanResponse {
            plan_id: plan_id.as_i32(),
            name: command.name,
            created_at: plan.created_at(),
        })
    }
}
