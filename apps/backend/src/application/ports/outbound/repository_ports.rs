use async_trait::async_trait;

use crate::domain::shared_kernel::value_objects::{UserId, SessionId, Email};
/// User Repository Port - defines interface for user data access
#[async_trait]
pub trait UserRepository: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    async fn find_by_id(&self, user_id: &UserId) -> Result<Option<User>, Self::Error>;
    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, Self::Error>;
    async fn save(&self, user: &User) -> Result<(), Self::Error>;
    async fn delete(&self, user_id: &UserId) -> Result<(), Self::Error>;
    async fn list_users(&self, offset: usize, limit: usize) -> Result<Vec<User>, Self::Error>;
}

/// Session Repository Port
#[async_trait]
pub trait SessionRepository: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    async fn find_by_id(&self, session_id: &SessionId) -> Result<Option<Session>, Self::Error>;
    async fn find_byuser_id(&self, user_id: &UserId) -> Result<Vec<Session>, Self::Error>;
    async fn save(&self, session: &Session) -> Result<(), Self::Error>;
    async fn delete(&self, session_id: &SessionId) -> Result<(), Self::Error>;
}

/// Audit Repository Port
#[async_trait]
pub trait AuditRepository: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    async fn log_event(&self, event: &dyn AuditEvent) -> Result<(), Self::Error>;
    async fn find_events_by_user(&self, user_id: &UserId) -> Result<Vec<Box<dyn AuditEvent>>, Self::Error>;
}

/// User Permission Repository Port
#[async_trait]
pub trait UserPermissionRepository: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;
    
    async fn get_user_permissions(&self, user_id: &UserId) -> Result<Vec<String>, Self::Error>;
    async fn set_user_permissions(&self, user_id: &UserId, permissions: &[String]) -> Result<(), Self::Error>;
    async fn add_user_permission(&self, user_id: &UserId, permission: &str) -> Result<(), Self::Error>;
    async fn remove_user_permission(&self, user_id: &UserId, permission: &str) -> Result<(), Self::Error>;
    async fn has_permission(&self, user_id: &UserId, permission: &str) -> Result<bool, Self::Error>;
}

/// User search filters for repository queries
#[derive(Debug, Clone)]
pub struct UserSearchFilters {
    pub email_contains: Option<String>,
    pub is_active: Option<bool>,
    pub package_tier: Option<String>,
    pub created_after: Option<chrono::DateTime<chrono::Utc>>,
    pub last_login_after: Option<chrono::DateTime<chrono::Utc>>,
}

// Re-export domain types for convenience
pub use crate::domain::user_management::aggregates::{user::User, session::Session};

// Trait alias for audit events
pub trait AuditEvent: Send + Sync + std::fmt::Debug {
    fn event_type(&self) -> &'static str;
    fn user_id(&self) -> Option<String>;
    fn timestamp(&self) -> chrono::DateTime<chrono::Utc>;
}