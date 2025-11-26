//! OpenAPI Documentation Module
//!
//! This module provides API documentation using Scalar with utoipa.
//! Minimal OpenAPI spec for successfully migrated handlers only.

pub mod routes;
pub mod schemas;
pub mod openapi;

// Re-export main components
pub use openapi::ApiDoc;
pub use routes::{
    create_docs_routes,
    docs_scalar_handler,
    openapi_json_handler
};