use crate::prelude::*;
use crate::application::shared::{CommandHandler, ApplicationResult, ApplicationError};
use crate::application::permission_management::commands::{
    AssignWalletToGroupCommand, AssignWalletToGroupResponse
};
use crate::domain::permission_management::{
    PermissionGroupRepositoryPort, GroupAssignmentRepositoryPort, GroupId,
    domain_services::GroupAssignmentService
};
use crate::domain::wallet_management::WalletAddress;
use crate::domain::shared_kernel::DomainEventBus;

/// Command handler for assigning wallets to groups
pub struct AssignWalletToGroupCommandHandler {
    group_repository: Arc<dyn PermissionGroupRepositoryPort>,
    assignment_repository: Arc<dyn GroupAssignmentRepositoryPort>,
    _event_bus: Arc<dyn DomainEventBus>,
}

impl AssignWalletToGroupCommandHandler {
    pub fn new(
        group_repository: Arc<dyn PermissionGroupRepositoryPort>,
        assignment_repository: Arc<dyn GroupAssignmentRepositoryPort>,
        event_bus: Arc<dyn DomainEventBus>,
    ) -> Self {
        Self {
            group_repository,
            assignment_repository,
            _event_bus: event_bus,
        }
    }
}

#[async_trait]
impl CommandHandler<AssignWalletToGroupCommand> for AssignWalletToGroupCommandHandler {
    async fn handle(&self, command: AssignWalletToGroupCommand) -> ApplicationResult<AssignWalletToGroupResponse> {
        // 1. Parse group ID and wallet address
        let group_id = GroupId::from_str(&command.group_id)
            .map_err(|e| ApplicationError::validation("group_id", e.to_string()))?;

        let wallet_address = WalletAddress::new(&command.wallet_address)
            .map_err(|e| ApplicationError::validation("wallet_address", e.to_string()))?;

        let assigned_by = if let Some(addr) = &command.assigned_by {
            Some(WalletAddress::new(addr)
                .map_err(|e| ApplicationError::validation("assigned_by", e.to_string()))?)
        } else {
            None
        };

        // 2. Find group
        let group = self.group_repository.find_by_id(&group_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("PermissionGroup", command.group_id.clone()))?;

        // 3. Check member count
        let member_count = self.assignment_repository.count_group_members(&group_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 4. Validate assignment using domain service
        GroupAssignmentService::can_assign_wallet_to_group(&group, member_count)
            .map_err(ApplicationError::from)?;

        // 5. Create assignment
        let assignment = GroupAssignmentService::create_assignment(
            group_id,
            wallet_address.clone(),
            assigned_by,
            command.expires_at,
        );

        let assigned_at = chrono::Utc::now();

        // 6. Save assignment
        self.assignment_repository.save(&assignment).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 7. Return response
        Ok(AssignWalletToGroupResponse {
            group_id: command.group_id,
            wallet_address: wallet_address.as_str().to_string(),
            assigned_at,
        })
    }
}
