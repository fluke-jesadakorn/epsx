// Unified Route Architecture - Single source of truth for all routes
// Consolidates all competing router systems into one

pub mod unified_router;

// Re-export the unified route builder as the main interface
pub use unified_router::UnifiedRouteBuilder;