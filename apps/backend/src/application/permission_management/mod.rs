// Permission Management Application Layer
// Commands and queries for permission plan and policy operations

pub mod commands;
pub mod controllers; // HTTP controllers (inbound adapters)
pub mod dtos;
pub mod queries; // Request/Response DTOs

// Re-export command models
pub use commands::{
    AssignWalletToPlanCommand, AssignWalletToPlanResponse, CreatePermissionPlanCommand,
    CreatePermissionPlanResponse, DeletePermissionPlanCommand, DeletePermissionPlanResponse,
    RemoveWalletFromPlanCommand, RemoveWalletFromPlanResponse, UpdatePermissionPlanCommand,
    UpdatePermissionPlanResponse,
};

// Re-export command handlers
pub use commands::{
    AssignWalletToPlanCommandHandler, CreatePermissionPlanCommandHandler,
    DeletePermissionPlanCommandHandler, RemoveWalletFromPlanCommandHandler,
    UpdatePermissionPlanCommandHandler,
};

// Re-export query models
pub use queries::{
    GetPermissionPlanQuery, GetPermissionPlanResponse, GetPlanMembersQuery, GetPlanMembersResponse,
    GetWalletPlansQuery, GetWalletPlansResponse, ListPermissionPlansQuery,
    ListPermissionPlansResponse,
};

// Re-export query handlers
pub use queries::{
    GetPermissionPlanQueryHandler, GetPlanMembersQueryHandler, GetWalletPlansQueryHandler,
    ListPermissionPlansQueryHandler,
};
