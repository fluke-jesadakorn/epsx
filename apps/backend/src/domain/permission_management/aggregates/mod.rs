// Permission Management Aggregates

pub mod plan;
pub mod policy;

// Backward compatibility: re-export plan as permission_plan
pub mod permission_plan {
    pub use super::plan::*;
}

// Re-export from new file (plan.rs)
pub use plan::{
    Plan,
    CreatePlanParams,
    LoadPlanParams,
    UpdatePlanParams,
    // Backward compatibility aliases
    PermissionPlan,
    CreatePermissionPlanParams,
    LoadPermissionPlanParams,
    UpdatePermissionPlanParams,
};
pub use policy::Policy;
