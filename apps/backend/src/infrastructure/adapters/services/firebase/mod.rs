// Firebase service integrations

pub mod firebase_admin;
pub mod types;

// Re-export for convenience
pub use firebase_admin::*;
pub use types::*;