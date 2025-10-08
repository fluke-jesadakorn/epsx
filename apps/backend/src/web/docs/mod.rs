//! OpenAPI Documentation Module
//! 
//! This module provides comprehensive API documentation using utoipa and ReDoc.
//! The documentation is automatically generated from code annotations and provides
//! an interactive interface for developers.

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