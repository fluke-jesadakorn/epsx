// Permission Management Application Layer
// Commands and queries for permission plan and policy operations

pub mod commands;
pub mod queries;
pub mod controllers; // HTTP controllers (inbound adapters)
pub mod dtos; // Request/Response DTOs

// Re-export command models
pub use commands::{
    CreatePermissionPlanCommand,
    CreatePermissionPlanResponse,
    UpdatePermissionPlanCommand,
    UpdatePermissionPlanResponse,
    DeletePermissionPlanCommand,
    DeletePermissionPlanResponse,
    AssignWalletToPlanCommand,
    AssignWalletToPlanResponse,
    RemoveWalletFromPlanCommand,
    RemoveWalletFromPlanResponse,
};

// Re-export command handlers
pub use commands::{
    CreatePermissionPlanCommandHandler,
    UpdatePermissionPlanCommandHandler,
    DeletePermissionPlanCommandHandler,
    AssignWalletToPlanCommandHandler,
    RemoveWalletFromPlanCommandHandler,
};

// Re-export query models
pub use queries::{
    GetPermissionPlanQuery,
    GetPermissionPlanResponse,
    ListPermissionPlansQuery,
    ListPermissionPlansResponse,
    GetPlanMembersQuery,
    GetPlanMembersResponse,
    GetWalletPlansQuery,
    GetWalletPlansResponse,
};

// Re-export query handlers
pub use queries::{
    GetPermissionPlanQueryHandler,
    ListPermissionPlansQueryHandler,
    GetPlanMembersQueryHandler,
    GetWalletPlansQueryHandler,
};
