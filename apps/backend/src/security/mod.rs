// Security module for comprehensive security management
// permission_security removed - replaced by auth/roles.rs
pub mod brute_force;
pub mod brute_force_integration;
pub mod alerts;
pub mod webhooks;

// SecuritySummary removed - replaced by simple roles
pub use brute_force_integration::BruteForceIntegrationService;
// pub use brute_force::*;
// pub use alerts::*;
// pub use webhooks::*;