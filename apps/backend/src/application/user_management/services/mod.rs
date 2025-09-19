// User Management Application Services
// These orchestrate multiple domain operations and handle cross-cutting concerns

pub mod user_application_service;
pub mod user_query_service;
pub mod user_reference_resolver;

pub use user_application_service::UserApplicationService;
pub use user_query_service::UserQueryService;
pub use user_reference_resolver::{UserReferenceResolver, UserResolutionCapable};