use crate::prelude::*;
use crate::domain::permission_management::{PermissionPlan, PlanId, entities::PlanAssignment};
use crate::domain::wallet_management::WalletAddress;

/// Domain service for plan assignment logic
pub struct PlanAssignmentService;

impl PlanAssignmentService {
    /// Validate if a wallet can be assigned to a plan
    pub fn can_assign_wallet_to_plan(
        plan: &PermissionPlan,
        current_member_count: i64,
    ) -> AppResult<()> {
        // Check if plan is active
        if !plan.is_active() {
            return Err(AppError::validation_error(
                "Cannot assign wallet to inactive plan"
            ));
        }

        // Check member limit
        if let Some(max_members) = plan.max_members() {
            if current_member_count >= max_members as i64 {
                return Err(AppError::validation_error(
                    format!("Plan has reached maximum member limit of {}", max_members)
                ));
            }
        }

        Ok(())
    }

    /// Create a new plan assignment
    pub fn create_assignment(
        plan_id: PlanId,
        wallet_address: WalletAddress,
        assigned_by: Option<WalletAddress>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> PlanAssignment {
        PlanAssignment::new(plan_id, wallet_address, assigned_by, expires_at)
    }

    /// Check if an assignment is still valid
    pub fn is_assignment_valid(assignment: &PlanAssignment) -> bool {
        assignment.is_active()
    }
}
