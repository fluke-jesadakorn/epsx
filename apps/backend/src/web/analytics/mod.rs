// Analytics Module - Lightweight coordinator
// Focused modules split into separate files for better organization

pub mod eps_handlers;
pub mod eps;
pub mod repository;
pub mod websocket_service;
pub mod types;
pub mod admin_handlers;

// Re-exports
pub use repository::TradingViewEPSRepository;
pub use websocket_service::WebSocketEarningsService;
pub use types::{AuthenticatedUser, AnalyticsQuery};
pub use eps_handlers::*;
pub use admin_handlers::*;

// NOTE: Legacy create_analytics_router function DELETED
// All routes are now managed by UnifiedRouteBuilder in src/web/routes/unified_router.rs
// This function was creating duplicate routes and is no longer used.
// Deleted on: 2025-01-XX during route reconciliation cleanup
