// Session Management Bounded Context
// Handles session persistence, lifecycle, and cross-session operations
// Separate from Authentication which focuses on identity verification

pub mod aggregates;
pub mod value_objects;
// TODO: Implement these session management modules as needed
// pub mod domain_services;
// pub mod ports;
// pub mod events;
pub mod repositories;

// Re-export domain concepts
pub use aggregates::*;
pub use value_objects::*;
// TODO: Re-enable these exports once modules are implemented
// pub use domain_services::*;
// pub use ports::*;
// pub use events::*;
pub use repositories::*;

/// Session Management bounded context business rules and invariants
pub struct SessionManagementBoundedContext;

impl SessionManagementBoundedContext {
    /// Session management business rules
    pub const MAX_CONCURRENT_SESSIONS_PER_USER: u32 = 10;
    pub const SESSION_CLEANUP_BATCH_SIZE: u32 = 100;
    pub const EXPIRED_SESSION_RETENTION_DAYS: u32 = 30;
    pub const SESSION_ACTIVITY_TRACKING_INTERVAL_MINUTES: u32 = 5;
    
    /// Session security policies
    pub const MAX_INACTIVE_SESSION_HOURS: u32 = 24;
    pub const SUSPICIOUS_SESSION_THRESHOLD_SCORE: f64 = 75.0;
    pub const CONCURRENT_SESSION_WARNING_THRESHOLD: u32 = 5;
    
    /// Session data management
    pub const SESSION_METADATA_MAX_SIZE_KB: u32 = 64;
    pub const SESSION_HISTORY_RETENTION_DAYS: u32 = 90;
}