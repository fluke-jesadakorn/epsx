// User Management Application Layer
// This module contains the application logic for user management operations
// following CQRS and hexagonal architecture patterns

pub mod commands;
pub mod queries;
pub mod services;

// Re-export command and query models for easy access
pub use commands::{
    CreateUserCommand,
    CreateUserResponse,
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
    GetUserByFirebaseUidQuery,
    GetUserByFirebaseUidResponse,
    SearchUsersQuery,
    SearchUsersResponse,
    ListUsersQuery,
    ListUsersResponse,
    UserSummary,
};

// Re-export handlers
pub use commands::{
    CreateUserCommandHandler,
    UpdateUserCommandHandler,
    DeleteUserCommandHandler,
    GrantPermissionCommandHandler,
    CreateSessionCommandHandler,
};