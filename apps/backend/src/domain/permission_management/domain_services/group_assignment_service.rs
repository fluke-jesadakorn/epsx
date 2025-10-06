use crate::prelude::*;
use crate::domain::permission_management::{PermissionGroup, GroupId, entities::GroupAssignment};
use crate::domain::wallet_management::WalletAddress;

/// Domain service for group assignment logic
pub struct GroupAssignmentService;

impl GroupAssignmentService {
    /// Validate if a wallet can be assigned to a group
    pub fn can_assign_wallet_to_group(
        group: &PermissionGroup,
        current_member_count: i64,
    ) -> AppResult<()> {
        // Check if group is active
        if !group.is_active() {
            return Err(AppError::validation_error(
                "Cannot assign wallet to inactive group"
            ));
        }

        // Check member limit
        if let Some(max_members) = group.max_members() {
            if current_member_count >= max_members as i64 {
                return Err(AppError::validation_error(
                    format!("Group has reached maximum member limit of {}", max_members)
                ));
            }
        }

        Ok(())
    }

    /// Create a new group assignment
    pub fn create_assignment(
        group_id: GroupId,
        wallet_address: WalletAddress,
        assigned_by: Option<WalletAddress>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> GroupAssignment {
        GroupAssignment::new(group_id, wallet_address, assigned_by, expires_at)
    }

    /// Check if an assignment is still valid
    pub fn is_assignment_valid(assignment: &GroupAssignment) -> bool {
        assignment.is_active()
    }
}
