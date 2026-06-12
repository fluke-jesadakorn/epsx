//! `core` module for `epsx-identity-shared`.
//!
//! Contains:
//!   - `AppError` shim used by the moved auth source files
//!   - `permissions` rule store (CLAUDE.md "Permissions & Plan Logic
//!     — Backend Only" — this MUST stay callable in-process from the
//!     backend binary. The real `apps/backend/src/core/permissions.rs`
//!     remains the canonical implementation; the copy here is a
//!     literal duplicate of the rule logic so the moved auth source
//!     compiles standalone.)
//!
//! In a later wave the backend can `pub use
//! epsx_identity_shared::core::permissions::*;` and delete its local
//! copy.

pub mod permissions;

use serde::{Serialize, Deserialize};
use thiserror::Error;

pub type AppResult<T> = std::result::Result<T, AppError>;

#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("Validation error in field {field}: {message}")]
    ValidationField { field: String, message: String },
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    #[error("Authorization error: {0}")]
    AuthorizationError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

impl AppError {
    pub fn database_error<S: Into<String>>(msg: S) -> Self {
        Self::DatabaseError(msg.into())
    }
    pub fn not_found<S: Into<String>>(msg: S) -> Self {
        Self::NotFound(msg.into())
    }
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::ValidationError(msg.into())
    }
    pub fn validation_error<S: Into<String>>(msg: S) -> Self {
        Self::ValidationError(msg.into())
    }
    pub fn internal<S: Into<String>>(msg: S) -> Self {
        Self::InternalError(msg.into())
    }
    pub fn authentication<S: Into<String>>(msg: S) -> Self {
        Self::AuthenticationError(msg.into())
    }
    pub fn authorization<S: Into<String>>(msg: S) -> Self {
        Self::AuthorizationError(msg.into())
    }
    pub fn configuration<S: Into<String>>(msg: S) -> Self {
        Self::ConfigurationError(msg.into())
    }
}
