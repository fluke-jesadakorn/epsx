// Application Ports
// These define the interfaces for inbound (driving) and outbound (driven) interactions

pub mod inbound;
pub mod outbound;

// Re-export common port types
pub use outbound::*;

// Convenience re-exports for specific modules
pub mod repositories {
    pub use super::outbound::{WalletUserRepository, SessionRepository, AuditRepository, WalletUserPermissionRepository, WalletUserSearchFilters};
}

pub mod services {
    // Email service exports removed - Web3-first system uses direct wallet notifications
    pub use super::outbound::{NotificationServicePort};
}