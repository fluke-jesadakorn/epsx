use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::permission_management::commands::{
    DeletePermissionPlanCommand, DeletePermissionPlanResponse
};
use crate::domain::permission_management::{PermissionPlanRepositoryPort, PlanId, events::PlanDeletedEvent};
// wave11(track-c) R7: migrated from `Arc<dyn DomainEventBus>` to the
// kernel-level `EventPublisherPort`. The publish is async on the
// port (was sync on the bus). The `Box<dyn DomainEvent>` shape lets
// the port own the event across the await boundary.
use epsx_contracts::event_publisher_port::EventPublisherPort;

/// Command handler for deleting permission plans
pub struct DeletePermissionPlanCommandHandler {
    plan_repository: Arc<dyn PermissionPlanRepositoryPort>,
    event_publisher: Arc<dyn EventPublisherPort>,
}

impl DeletePermissionPlanCommandHandler {
    pub fn new(
        plan_repository: Arc<dyn PermissionPlanRepositoryPort>,
        event_publisher: Arc<dyn EventPublisherPort>,
    ) -> Self {
        Self {
            plan_repository,
            event_publisher,
        }
    }
}

#[async_trait]
impl CommandHandler<DeletePermissionPlanCommand> for DeletePermissionPlanCommandHandler {
    async fn handle(&self, command: DeletePermissionPlanCommand) -> ApplicationResult<DeletePermissionPlanResponse> {
        // 1. Parse plan ID
        let plan_id = PlanId::parse(&command.plan_id)
            .map_err(|e| ApplicationError::validation("plan_id", e.to_string()))?;

        // 2. Check if plan exists
        let _plan = self.plan_repository.find_by_id(&plan_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("PermissionPlan", command.plan_id.clone()))?;

        // 3. Delete plan
        self.plan_repository.delete(&plan_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 4. Publish PlanDeletedEvent (R7 + R8 wiring — was _event_bus
        //    before wave 10; routed through the new EventPublisherPort
        //    in wave 11). The in-process adapter is a no-op stub (logs
        //    at tracing::info!); no real consumer exists today.
        let event = PlanDeletedEvent::new(
            plan_id.as_str().to_string(),
            0, // aggregate version not tracked per-delete; event-sourcing build fills this
            plan_id.as_str().to_string(),
            Utc::now(),
        );
        let event_box: Box<dyn crate::domain::shared_kernel::DomainEvent> = Box::new(event);
        if let Err(e) = self.event_publisher.publish(event_box).await {
            // Publish failures must not break the command — the
            // in-process adapter is infallible, but a future
            // network impl (wave-N+2) might return
            // `AppError::Infrastructure` on transport failures.
            // Log loudly and continue.
            tracing::warn!(
                error = %e,
                event = "PlanDeletedEvent",
                "EventPublisherPort.publish returned error; command continues"
            );
        }

        // 5. Return response
        Ok(DeletePermissionPlanResponse {
            plan_id: command.plan_id,
            deleted: true,
        })
    }
}
