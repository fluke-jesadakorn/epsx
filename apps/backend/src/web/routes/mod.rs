// Simple Route Architecture - Single source of truth for all routes
// Eliminates complex contextual routing and over-engineering

pub mod simple_routes;

// Legacy unified routes (deprecated - will be removed)
// #[deprecated(note = "Use simple_routes::SimpleRouteBuilder instead")]
// pub mod unified; // Removed - compilation issues

// Re-export the simple route builder as the main interface
pub use simple_routes::SimpleRouteBuilder;