// Domain services - business logic that doesn't naturally fit in entities

pub mod role_hierarchy;
// pub mod permission_checker;
pub mod audit_service;
pub mod auto_assignment;
pub mod feature_expiration;
pub mod permission_resolver;
pub mod permission_cache_service;
pub mod permissions;
// casbin_service removed - using modern JWT auth
pub mod eps_ranking_service;
pub mod eps_cache_service;
pub mod firebase_user_service;
pub mod firebase_session_service;
pub mod database_role_service;
pub mod admin_module_service;

// Temporary glob re-exports - to be refined in next phase
// pub use role_hierarchy::*;
pub use audit_service::*;
pub use auto_assignment::*;
pub use feature_expiration::*;
pub use permission_resolver::*;
pub use permission_cache_service::*;
pub use permissions::*;
pub use eps_ranking_service::*;
pub use eps_cache_service::*;
pub use firebase_user_service::*;
pub use firebase_session_service::*;
pub use database_role_service::*;
pub use admin_module_service::*;