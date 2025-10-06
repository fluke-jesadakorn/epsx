// Permission Management Application Layer
// Commands and queries for permission group and policy operations

pub mod commands;
pub mod queries;
pub mod controllers; // HTTP controllers (inbound adapters)
pub mod dtos; // Request/Response DTOs

// Re-export command models
pub use commands::{
    CreatePermissionGroupCommand,
    CreatePermissionGroupResponse,
    UpdatePermissionGroupCommand,
    UpdatePermissionGroupResponse,
    DeletePermissionGroupCommand,
    DeletePermissionGroupResponse,
    AssignWalletToGroupCommand,
    AssignWalletToGroupResponse,
    RemoveWalletFromGroupCommand,
    RemoveWalletFromGroupResponse,
};

// Re-export command handlers
pub use commands::{
    CreatePermissionGroupCommandHandler,
    UpdatePermissionGroupCommandHandler,
    DeletePermissionGroupCommandHandler,
    AssignWalletToGroupCommandHandler,
    RemoveWalletFromGroupCommandHandler,
};

// Re-export query models
pub use queries::{
    GetPermissionGroupQuery,
    GetPermissionGroupResponse,
    ListPermissionGroupsQuery,
    ListPermissionGroupsResponse,
    GetGroupMembersQuery,
    GetGroupMembersResponse,
    GetWalletGroupsQuery,
    GetWalletGroupsResponse,
};

// Re-export query handlers
pub use queries::{
    GetPermissionGroupQueryHandler,
    ListPermissionGroupsQueryHandler,
    GetGroupMembersQueryHandler,
    GetWalletGroupsQueryHandler,
};
