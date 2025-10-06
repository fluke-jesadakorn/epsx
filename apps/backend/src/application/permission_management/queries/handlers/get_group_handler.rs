use crate::prelude::*;
use crate::application::shared::{QueryHandler, ApplicationResult, ApplicationError};
use crate::application::permission_management::queries::{
    GetPermissionGroupQuery, GetPermissionGroupResponse
};
use crate::domain::permission_management::{PermissionGroupRepositoryPort, GroupAssignmentRepositoryPort, GroupId};

/// Query handler for getting a single permission group
pub struct GetPermissionGroupQueryHandler {
    group_repository: Arc<dyn PermissionGroupRepositoryPort>,
    assignment_repository: Arc<dyn GroupAssignmentRepositoryPort>,
}

impl GetPermissionGroupQueryHandler {
    pub fn new(
        group_repository: Arc<dyn PermissionGroupRepositoryPort>,
        assignment_repository: Arc<dyn GroupAssignmentRepositoryPort>,
    ) -> Self {
        Self {
            group_repository,
            assignment_repository,
        }
    }
}

#[async_trait]
impl QueryHandler<GetPermissionGroupQuery> for GetPermissionGroupQueryHandler {
    async fn handle(&self, query: GetPermissionGroupQuery) -> ApplicationResult<GetPermissionGroupResponse> {
        // 1. Parse group ID
        let group_id = GroupId::from_str(&query.group_id)
            .map_err(|e| ApplicationError::validation("group_id", e.to_string()))?;

        // 2. Find group
        let group = self.group_repository.find_by_id(&group_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?
            .ok_or_else(|| ApplicationError::not_found("PermissionGroup", query.group_id))?;

        // 3. Get member count
        let member_count = self.assignment_repository.count_group_members(&group_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 4. Return response
        Ok(GetPermissionGroupResponse {
            id: group.id().as_str(),
            name: group.name().to_string(),
            slug: group.slug().as_str().to_string(),
            description: group.description().to_string(),
            group_type: group.group_type().to_string(),
            permissions: group.permissions().iter().map(|p| p.as_str().to_string()).collect(),
            price: group.price(),
            currency: group.currency().to_string(),
            billing_cycle: group.billing_cycle().to_string(),
            is_active: group.is_active(),
            is_promoted: group.is_promoted(),
            display_order: group.display_order(),
            max_members: group.max_members(),
            auto_assign_enabled: group.auto_assign_enabled(),
            metadata: group.metadata().clone(),
            created_at: group.created_at(),
            updated_at: group.updated_at(),
            member_count,
        })
    }
}
