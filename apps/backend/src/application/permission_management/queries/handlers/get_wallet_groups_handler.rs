use crate::prelude::*;
use crate::application::shared::{QueryHandler, ApplicationResult, ApplicationError};
use crate::application::permission_management::queries::{
    GetWalletGroupsQuery, GetWalletGroupsResponse, WalletGroupInfo
};
use crate::domain::permission_management::{GroupAssignmentRepositoryPort, PermissionGroupRepositoryPort};
use crate::domain::wallet_management::WalletAddress;

/// Query handler for getting wallet groups
pub struct GetWalletGroupsQueryHandler {
    assignment_repository: Arc<dyn GroupAssignmentRepositoryPort>,
    group_repository: Arc<dyn PermissionGroupRepositoryPort>,
}

impl GetWalletGroupsQueryHandler {
    pub fn new(
        assignment_repository: Arc<dyn GroupAssignmentRepositoryPort>,
        group_repository: Arc<dyn PermissionGroupRepositoryPort>,
    ) -> Self {
        Self {
            assignment_repository,
            group_repository,
        }
    }
}

#[async_trait]
impl QueryHandler<GetWalletGroupsQuery> for GetWalletGroupsQueryHandler {
    async fn handle(&self, query: GetWalletGroupsQuery) -> ApplicationResult<GetWalletGroupsResponse> {
        // 1. Parse wallet address
        let wallet_address = WalletAddress::new(&query.wallet_address)
            .map_err(|e| ApplicationError::validation("wallet_address", e.to_string()))?;

        // 2. Find assignments
        let assignments = self.assignment_repository.find_by_wallet(&wallet_address).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 3. Get group details for each assignment with actual timestamps
        let mut groups = Vec::new();
        for assignment in assignments {
            if let Ok(Some(group)) = self.group_repository.find_by_id(assignment.group_id()).await {
                groups.push(WalletGroupInfo {
                    group_id: group.id().as_str(),
                    group_name: group.name().to_string(),
                    group_slug: group.slug().as_str().to_string(),
                    permissions: group.permissions().iter().map(|p| p.as_str().to_string()).collect(),
                    assigned_at: assignment.assigned_at(),
                    expires_at: assignment.expires_at(),
                    is_active: assignment.is_active(),
                });
            }
        }

        Ok(GetWalletGroupsResponse {
            wallet_address: query.wallet_address,
            total: groups.len() as i64,
            groups,
        })
    }
}
