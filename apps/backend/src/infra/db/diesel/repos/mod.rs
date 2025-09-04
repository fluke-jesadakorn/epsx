pub mod user_repo;
pub mod user_permission_repo;
pub mod user_dynamic_limit_repo;
pub mod session_repo;
pub mod audit_repo;
pub mod usage_repo;
pub mod refresh_token_repo;
pub mod revoked_token_repo;
pub mod user_notification_repo;

// Re-export all repositories for convenience
pub use user_repo::DieselUserRepository;
pub use user_permission_repo::DieselUserPermissionRepository;
pub use user_dynamic_limit_repo::{UserDynamicLimitRepository, DynamicLimitAssignmentBuilder};
pub use session_repo::DieselSessionRepository;
pub use audit_repo::DieselAuditRepository;
pub use usage_repo::{DieselUsageRepository, StubUsageRepository, StubStockRepository};
pub use refresh_token_repo::RefreshTokenRepository;
pub use revoked_token_repo::{RevokedTokenRepository, RevocationStats};
pub use user_notification_repo::{UserNotificationRepository, UserNotificationWithDetails, AdminNotificationStats};