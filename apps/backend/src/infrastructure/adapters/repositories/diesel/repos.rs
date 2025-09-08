// Repository implementations using Diesel ORM
pub use super::super::{
    user_repository_adapter::*, 
    session_repository_adapter::*,
    notification_repository_adapter::*,
};

// Type aliases for legacy compatibility
pub type UserNotificationRepository = super::super::notification_repository_adapter::NotificationRepositoryAdapter;
pub type DieselUserRepository = super::super::user_repository_adapter::UserRepositoryAdapter;
pub type DieselSessionRepository = super::super::session_repository_adapter::SessionRepositoryAdapter;
pub type DieselUserPermissionRepository = super::super::user_repository_adapter::UserRepositoryAdapter;
pub type RevokedTokenRepository = super::super::user_repository_adapter::UserRepositoryAdapter;
pub type RefreshTokenRepository = super::super::user_repository_adapter::UserRepositoryAdapter;

// Module re-exports for compatibility
pub mod user_repo {
    pub use super::DieselUserRepository;
}