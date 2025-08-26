// Clean architecture library exports
#![allow(dead_code)]

pub mod core;       // Shared kernel
pub mod dom;        // Domain layer
pub mod app;        // Application layer
pub mod infra;      // Infrastructure layer
pub mod web;        // Web/API layer
pub mod config;     // Configuration
pub mod auth;       // Modern authentication
// permissions module removed - replaced by auth/roles.rs
pub mod stock;      // Stock management

// Selective re-exports for clean namespace
pub use core::{errors, types, telemetry};
pub use infra::AppContainer;
pub use web::create_router;