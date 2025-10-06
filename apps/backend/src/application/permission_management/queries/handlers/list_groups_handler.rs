use crate::prelude::*;
use crate::application::shared::{QueryHandler, ApplicationResult, ApplicationError};
use crate::application::permission_management::queries::{
    ListPermissionGroupsQuery, ListPermissionGroupsResponse, PermissionGroupSummary
};
use crate::domain::permission_management::{PermissionGroupRepositoryPort, GroupAssignmentRepositoryPort, repository_ports::GroupSearchCriteria};

/// Query handler for listing permission groups
pub struct ListPermissionGroupsQueryHandler {
    group_repository: Arc<dyn PermissionGroupRepositoryPort>,
    assignment_repository: Arc<dyn GroupAssignmentRepositoryPort>,
}

impl ListPermissionGroupsQueryHandler {
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
impl QueryHandler<ListPermissionGroupsQuery> for ListPermissionGroupsQueryHandler {
    async fn handle(&self, query: ListPermissionGroupsQuery) -> ApplicationResult<ListPermissionGroupsResponse> {
        let page = query.page.unwrap_or(1);
        let limit = query.limit.unwrap_or(20);
        let offset = ((page - 1) * limit) as i64;

        // 1. Build search criteria
        let criteria = GroupSearchCriteria {
            group_type: query.group_type,
            is_active: query.is_active,
            is_promoted: query.is_promoted,
            search_term: query.search_term,
            limit: Some(limit as i64),
            offset: Some(offset),
        };

        // 2. Find groups
        let groups = self.group_repository.find_all(criteria.clone()).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 3. Get total count
        let total = self.group_repository.count(criteria).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 4. Build summaries with member counts
        let mut summaries = Vec::new();
        for group in groups {
            let member_count = self.assignment_repository.count_group_members(group.id()).await
                .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

            summaries.push(PermissionGroupSummary {
                id: group.id().as_str(),
                name: group.name().to_string(),
                slug: group.slug().as_str().to_string(),
                description: group.description().to_string(),
                group_type: group.group_type().to_string(),
                permissions: group.permissions().iter().map(|p| p.as_str().to_string()).collect(),
                price: group.price(),
                currency: group.currency().to_string(),
                is_active: group.is_active(),
                is_promoted: group.is_promoted(),
                member_count,
            });
        }

        Ok(ListPermissionGroupsResponse {
            groups: summaries,
            total,
            page,
            limit,
        })
    }
}
