// Authentication Services  
// Domain services for secure token management and refresh

pub mod secure_refresh_service;
// pub mod threat_detection_service; // Removed - unused service

// New contextual authentication services
pub mod internal_auth_service;
pub mod external_auth_service;
pub mod web3_auth_service;
pub mod web3_permission_service;

// Pure Web3 authentication service (wallet-first)
pub mod pure_web3_auth_service;

pub use secure_refresh_service::*;
// pub use threat_detection_service::*; // Removed - unused service
pub use internal_auth_service::*;
pub use external_auth_service::*;
pub use web3_auth_service::*;
pub use web3_permission_service::*;
pub use pure_web3_auth_service::*;