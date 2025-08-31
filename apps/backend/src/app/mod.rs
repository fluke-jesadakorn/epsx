// Application layer - Use cases, ports, and application services

pub mod use_cases;
pub mod ports;
pub mod dtos;
pub mod services_legacy;
pub mod services;

// Selective re-exports for clean interfaces
pub use use_cases::{AuthUC, UserMgmtUC};
pub use ports::repositories::{UserRepository, SessionRepository, AuditRepository};
pub use dtos::auth::{LoginReq, LoginRes, LogoutReq};
pub use services::{AppService, PermissionApplicationService};