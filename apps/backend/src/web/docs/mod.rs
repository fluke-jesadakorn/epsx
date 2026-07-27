//! OpenAPI Documentation Module
//!
//! This module provides API documentation using Scalar with utoipa.
//! Separate documentation for users (/docs) and admins (/admin/docs).

pub mod openapi;
pub mod openapi_admin;
pub mod openapi_user;
pub mod routes;
pub mod schemas;

// Re-export main components
pub use openapi::ApiDoc;
pub use openapi_admin::AdminApiDoc;
pub use openapi_user::UserApiDoc;
pub use routes::{
    create_docs_routes,
    docs_admin_handler,
    // Backward compatibility
    docs_scalar_handler,
    docs_user_handler,
    openapi_admin_json_handler,
    openapi_json_handler,
    openapi_user_json_handler,
};
