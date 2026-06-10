//! Re-export of the standard Result alias bound to AppError.
//!
//! This is the canonical Result type that all services should use.

pub use crate::error::AppResult;
pub use std::result::Result as StdResult;
