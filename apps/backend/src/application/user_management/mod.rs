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
// Note: WalletQueryService removed - migrated to direct wallet-based repository access
// as part of Web3-first user management migration

// Re-export handlers
pub use commands::{
    UpdateUserCommandHandler,
    DeleteUserCommandHandler,
    GrantPermissionCommandHandler,
    CreateSessionCommandHandler,
};