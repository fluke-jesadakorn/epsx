// Application Services re-exports

// Re-export specific application services
pub use super::user_management::services::UserApplicationService;

// Permission service placeholder
pub struct PermissionApplicationService;

impl PermissionApplicationService {
    pub fn new() -> Self {
        Self
    }
}

/// Application-level permission errors
#[derive(Debug, thiserror::Error)]
pub enum ApplicationPermissionError {
    #[error("Permission denied: {message}")]
    AccessDenied { message: String },
    
    #[error("Invalid permission format: {permission}")]
    InvalidPermission { permission: String },
    
    #[error("Permission not found: {permission}")]
    PermissionNotFound { permission: String },
    
    #[error("User not found: {user_id}")]
    UserNotFound { user_id: String },
    
    #[error("Database error: {source}")]
    DatabaseError { source: Box<dyn std::error::Error + Send + Sync> },
    
    #[error("Internal service error: {message}")]
    InternalError { message: String },
}