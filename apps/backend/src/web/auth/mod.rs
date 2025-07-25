// Authentication web module

pub mod handlers;
pub mod multi_handlers;
pub mod routes;

pub use handlers::*;
pub use multi_handlers::*;
pub use routes::{auth_routes, auth_routes_v1, AppState};