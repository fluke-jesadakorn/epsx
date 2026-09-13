// Analytics Module - Lightweight coordinator
// Focused modules split into separate files for better organization

pub mod admin_handlers;
pub mod eps;
pub mod eps_handlers;
pub mod repository;
pub mod types;
pub mod websocket_service;

// Re-exports
pub use admin_handlers::*;
pub use eps_handlers::*;
pub use repository::TradingViewEPSRepository;
pub use types::{AnalyticsQuery, AuthenticatedUser};
pub use websocket_service::WebSocketEarningsService;

// NOTE: Legacy create_analytics_router function DELETED
// All routes are now managed by UnifiedRouteBuilder in src/web/routes/unified_router.rs
// This function was creating duplicate routes and is no longer used.
// Deleted on: 2025-01-XX during route reconciliation cleanup
