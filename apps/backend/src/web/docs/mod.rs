//! OpenAPI Documentation Module
//!
//! This module provides API documentation using Scalar with utoipa.
//! Separate documentation for users (/docs) and admins (/admin/docs).

pub mod routes;
pub mod schemas;
pub mod openapi;
pub mod openapi_user;
pub mod openapi_admin;

// Re-export main components
pub use openapi::ApiDoc;
pub use openapi_user::UserApiDoc;
pub use openapi_admin::AdminApiDoc;
pub use routes::{
    create_docs_routes,
    docs_user_handler,
    docs_admin_handler,
    openapi_user_json_handler,
    openapi_admin_json_handler,
    // Backward compatibility
    docs_scalar_handler,
    openapi_json_handler
};