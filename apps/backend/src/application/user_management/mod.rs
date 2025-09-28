// User Management Application Layer
// This module contains the application logic for user management operations
// following CQRS and hexagonal architecture patterns

pub mod commands;
pub mod queries;
pub mod services;

// Re-export command and query models for easy access
pub use commands::{
    UpdateUserCommand,
    UpdateUserResponse,
    DeleteUserCommand,
    DeleteUserResponse,
    GrantPermissionCommand,
    GrantPermissionResponse,
    CreateSessionCommand,
    CreateSessionResponse,
};

pub use queries::{
    GetUserQuery,
    GetUserResponse,
    SearchUsersQuery,
    SearchUsersResponse,
    ListUsersQuery,
    ListUsersResponse,
    UserSummary,
};

// Re-export services
pub use services::{
    // UserReferenceResolver,     // REMOVED - legacy user reference system
    // UserResolutionCapable,     // REMOVED - legacy user reference system
    // UserApplicationService,    // Removed - service deleted
    WalletQueryService,
};

// Re-export handlers
pub use commands::{
    UpdateUserCommandHandler,
    DeleteUserCommandHandler,
    GrantPermissionCommandHandler,
    CreateSessionCommandHandler,
};