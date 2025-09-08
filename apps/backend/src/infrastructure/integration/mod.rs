// Infrastructure Integration Layer
// Orchestrates DDD bounded contexts with existing legacy systems
// Maintains API compatibility while enabling DDD architecture internally

pub mod authentication_service_integration;
pub mod payment_service_integration;
pub mod realtime_events_service_integration;

pub use authentication_service_integration::{
    AuthenticationServiceIntegration, SessionCreationResult, SessionRefreshResult, 
    TokenRefreshResult, SessionValidationResult, UserProfile, AuthenticationError
};
pub use payment_service_integration::*;
pub use realtime_events_service_integration::*;