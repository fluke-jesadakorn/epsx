// Permission Management Aggregates

pub mod group;
pub mod policy;

// Backward compatibility: re-export group as permission_group
pub mod permission_group {
    pub use super::group::*;
}

// Re-export from new file (group.rs)
pub use group::{
    Group,
    CreateGroupParams,
    LoadGroupParams,
    UpdateGroupParams,
    // Backward compatibility aliases
    PermissionGroup,
    CreatePermissionGroupParams,
    LoadPermissionGroupParams,
    UpdatePermissionGroupParams,
};
pub use policy::Policy;
