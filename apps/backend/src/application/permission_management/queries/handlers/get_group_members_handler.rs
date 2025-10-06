use crate::prelude::*;
use crate::application::shared::{QueryHandler, ApplicationResult, ApplicationError};
use crate::application::permission_management::queries::{
    GetGroupMembersQuery, GetGroupMembersResponse, GroupMemberInfo
};
use crate::domain::permission_management::{GroupAssignmentRepositoryPort, GroupId};

/// Query handler for getting group members
pub struct GetGroupMembersQueryHandler {
    assignment_repository: Arc<dyn GroupAssignmentRepositoryPort>,
}

impl GetGroupMembersQueryHandler {
    pub fn new(assignment_repository: Arc<dyn GroupAssignmentRepositoryPort>) -> Self {
        Self {
            assignment_repository,
        }
    }
}

#[async_trait]
impl QueryHandler<GetGroupMembersQuery> for GetGroupMembersQueryHandler {
    async fn handle(&self, query: GetGroupMembersQuery) -> ApplicationResult<GetGroupMembersResponse> {
        // 1. Parse group ID
        let group_id = GroupId::from_str(&query.group_id)
            .map_err(|e| ApplicationError::validation("group_id", e.to_string()))?;

        // 2. Find assignments
        let assignments = self.assignment_repository.find_by_group(&group_id).await
            .map_err(|e| ApplicationError::infrastructure(e.to_string()))?;

        // 3. Build response
        let members: Vec<GroupMemberInfo> = assignments
            .iter()
            .map(|a| GroupMemberInfo {
                wallet_address: a.wallet_address().as_str().to_string(),
                assigned_at: chrono::Utc::now(), // TODO: Get from assignment
                expires_at: None, // TODO: Get from assignment
                is_active: a.is_active(),
            })
            .collect();

        Ok(GetGroupMembersResponse {
            group_id: query.group_id,
            total: members.len() as i64,
            members,
        })
    }
}
