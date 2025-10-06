use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::subscription_management::commands::{
    UpdatePlanCommand, UpdatePlanResponse
};
use crate::domain::subscription_management::{PlanId, PlanRepositoryPort, Price, BillingCycle};
use crate::domain::shared_kernel::DomainEventBus;
use rust_decimal::Decimal;
use std::str::FromStr;

/// Command handler for updating plans
pub struct UpdatePlanCommandHandler {
    plan_repository: Arc<dyn PlanRepositoryPort>,
    event_bus: Arc<dyn DomainEventBus>,
}

impl UpdatePlanCommandHandler {
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
impl CommandHandler<UpdatePlanCommand> for UpdatePlanCommandHandler {
    async fn handle(&self, command: UpdatePlanCommand) -> ApplicationResult<UpdatePlanResponse> {
        // 1. Parse plan ID
        let plan_id = PlanId::from_i32(command.plan_id);

        // 2. Load existing plan
        let mut plan = self.plan_repository.find_by_id(&plan_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("plan_id", "Plan not found"))?;

        // 3. Parse optional price if both price and currency provided
        let price_vo = if let (Some(price_val), Some(currency_val)) = (&command.price, &command.currency) {
            let price_decimal = Decimal::from_str(&price_val.to_string())
                .map_err(|e| ApplicationError::validation("price", format!("Invalid price format: {}", e)))?;
            Some(Price::new(price_decimal, currency_val.clone())
                .map_err(|e| ApplicationError::validation("price", e.to_string()))?)
        } else {
            None
        };

        // 4. Parse optional billing cycle
        let billing_cycle = if let Some(bc) = &command.billing_cycle {
            Some(BillingCycle::from_str(bc)
                .map_err(|e| ApplicationError::validation("billing_cycle", e.to_string()))?)
        } else {
            None
        };

        // 5. Update plan
        plan.update(
            command.name,
            command.description,
            price_vo,
            billing_cycle,
            None, // features - not in command
            command.target_audience,
            command.is_active,
            command.is_promoted,
            command.display_order,
            command.metadata,
        ).map_err(|e| ApplicationError::business_logic(e.to_string()))?;

        // 6. Save updated plan
        self.plan_repository.save(&plan).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 7. Publish domain events
        for event in plan.uncommitted_events() {
            self.event_bus.publish(&**event);
        }

        // 8. Return response
        Ok(UpdatePlanResponse {
            plan_id: command.plan_id,
            updated_at: plan.updated_at(),
        })
    }
}
