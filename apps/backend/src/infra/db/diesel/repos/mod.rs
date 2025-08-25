pub mod user_repo;
pub mod session_repo;
pub mod audit_repo;
pub mod stock_repo;
pub mod iam_repo;
pub mod permission_profile_repo;
pub mod temporary_permission_repo;
pub mod module_repo;
pub mod usage_repo;
pub mod eps_repo;

// Re-export all repositories for convenience
pub use user_repo::DieselUserRepo;
pub use session_repo::DieselSessionRepo;
pub use audit_repo::DieselAuditRepo;
pub use stock_repo::DieselStockRepo;
pub use iam_repo::DieselIamRepo;
pub use permission_profile_repo::DieselPermissionProfileRepo;
pub use temporary_permission_repo::{DieselTemporaryPermissionRepo, StubTemporaryPermissionRepo};
pub use module_repo::{DieselModuleRepo, StubModuleRepo};
pub use usage_repo::DieselUsageRepo;
pub use eps_repo::DieselEPSRepository;