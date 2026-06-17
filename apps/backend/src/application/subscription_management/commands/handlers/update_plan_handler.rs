use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::subscription_management::commands::{
    UpdatePlanCommand, UpdatePlanResponse
};
use crate::domain::subscription_management::{PlanId, PlanRepositoryPort, Price, BillingCycle};
use crate::domain::subscription_management::aggregates::UpdatePlanParams;
use epsx_contracts::event_publisher_port::EventPublisherPort;
use rust_decimal::Decimal;
use std::str::FromStr;

/// Command handler for updating plans
pub struct UpdatePlanCommandHandler {
    plan_repository: Arc<dyn PlanRepositoryPort>,
    event_publisher: Arc<dyn EventPublisherPort>,
}

impl UpdatePlanCommandHandler {
    pub fn new(
        plan_repository: Arc<dyn PlanRepositoryPort>,
        event_publisher: Arc<dyn EventPublisherPort>,
    ) -> Self {
        Self {
            plan_repository,
            event_publisher,
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
        plan.update(UpdatePlanParams {
            name: command.name,
            description: command.description,
            price: price_vo,
            billing_cycle,
            features: None, // features - not in command
            target_audience: command.target_audience,
            is_active: command.is_active,
            is_promoted: command.is_promoted,
            display_order: command.display_order,
            metadata: command.metadata,
        }).map_err(|e| ApplicationError::business_logic(e.to_string()))?;

        // 6. Save updated plan
        self.plan_repository.save(&plan).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 7. Publish domain events
        for event in plan.uncommitted_events() {
            let owned: Box<dyn crate::domain::shared_kernel::DomainEvent> = Box::new(epsx_contracts::domain_event::OwnedEvent::from_borrowed(&**event));
            if let Err(e) = self.event_publisher.publish(owned).await {
                tracing::warn!(
                    error = %e,
                    "EventPublisherPort.publish returned error; command continues"
                );
            }
        }

        // 8. Return response
        Ok(UpdatePlanResponse {
            plan_id: command.plan_id,
            updated_at: plan.updated_at(),
        })
    }
}
