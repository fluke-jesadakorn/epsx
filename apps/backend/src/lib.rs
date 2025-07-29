// Clean architecture library exports

pub mod core;   // Shared kernel
pub mod dom;    // Domain layer
pub mod app;    // Application layer
pub mod infra;  // Infrastructure layer
pub mod web;    // Web/API layer
pub mod config; // Configuration

// Re-exports for convenience
#[allow(ambiguous_glob_reexports)]
pub use core::*;
#[allow(ambiguous_glob_reexports)]
pub use dom::*;
#[allow(ambiguous_glob_reexports)]
pub use app::*;
#[allow(ambiguous_glob_reexports)]
pub use infra::*;
#[allow(ambiguous_glob_reexports)]
pub use web::*;