use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::permission_management::commands::{
    AssignWalletToPlanCommand, AssignWalletToPlanResponse
};
use crate::domain::permission_management::{
    PermissionPlanRepositoryPort, PlanAssignmentRepositoryPort, PlanId,
    domain_services::PlanAssignmentService
};
use crate::domain::wallet_management::WalletAddress;
use crate::domain::shared_kernel::DomainEventBus;

/// Command handler for assigning wallets to plans
pub struct AssignWalletToPlanCommandHandler {
    plan_repository: Arc<dyn PermissionPlanRepositoryPort>,
    assignment_repository: Arc<dyn PlanAssignmentRepositoryPort>,
    _event_bus: Arc<dyn DomainEventBus>,
}

impl AssignWalletToPlanCommandHandler {
    pub fn new(
        plan_repository: Arc<dyn PermissionPlanRepositoryPort>,
        assignment_repository: Arc<dyn PlanAssignmentRepositoryPort>,
        event_bus: Arc<dyn DomainEventBus>,
    ) -> Self {
        Self {
            plan_repository,
            assignment_repository,
            _event_bus: event_bus,
        }
    }
}

#[async_trait]
impl CommandHandler<AssignWalletToPlanCommand> for AssignWalletToPlanCommandHandler {
    async fn handle(&self, command: AssignWalletToPlanCommand) -> ApplicationResult<AssignWalletToPlanResponse> {
        // 1. Parse plan ID and wallet address
        let plan_id = PlanId::parse(&command.plan_id)
            .map_err(|e| ApplicationError::validation("plan_id", e.to_string()))?;

        let wallet_address = WalletAddress::new(&command.wallet_address)
            .map_err(|e| ApplicationError::validation("wallet_address", e.to_string()))?;

        let assigned_by = if let Some(addr) = &command.assigned_by {
            Some(WalletAddress::new(addr)
                .map_err(|e| ApplicationError::validation("assigned_by", e.to_string()))?)
        } else {
            None
        };

        // 2. Find plan
        let plan = self.plan_repository.find_by_id(&plan_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("PermissionPlan", command.plan_id.clone()))?;

        // 3. Check member count
        let member_count = self.assignment_repository.count_plan_members(&plan_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 4. Validate assignment using domain service
        PlanAssignmentService::can_assign_wallet_to_plan(&plan, member_count)
            .map_err(ApplicationError::from)?;

        // 5. Create assignment
        let assignment = PlanAssignmentService::create_assignment(
            plan_id,
            wallet_address.clone(),
            assigned_by,
            command.expires_at,
        );

        let assigned_at = chrono::Utc::now();

        // 6. Save assignment
        self.assignment_repository.save(&assignment).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 7. Return response
        Ok(AssignWalletToPlanResponse {
            plan_id: command.plan_id,
            wallet_address: wallet_address.as_str().to_string(),
            assigned_at,
        })
    }
}
