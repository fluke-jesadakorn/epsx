// Re-export authentication bounded context
pub use super::authentication::*;

/// Authentication Use Case - legacy compatibility
pub struct AuthUC;

impl AuthUC {
    pub fn new() -> Self {
        Self
    }
}