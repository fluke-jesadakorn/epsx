// Admin module for user management endpoints

pub mod handlers;
pub mod routes;

pub use routes::{create_admin_routes, create_admin_public_routes};