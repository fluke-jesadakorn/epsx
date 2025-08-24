// Clean architecture library exports

pub mod core;       // Shared kernel
pub mod dom;        // Domain layer
pub mod app;        // Application layer
pub mod infra;      // Infrastructure layer
pub mod web;        // Web/API layer
pub mod config;     // Configuration
pub mod auth;       // Modern authentication
pub mod security;   // Security layer
pub mod permissions; // Permission system
pub mod payment;    // Payment handling
pub mod stock;      // Stock management

// Selective re-exports for clean namespace
pub use core::{errors, types, telemetry};
pub use infra::{AppContainer, DieselAuditRepo};
pub use web::create_router;