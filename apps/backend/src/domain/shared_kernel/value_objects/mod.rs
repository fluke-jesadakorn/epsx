// Shared Value Objects - Common types used across bounded contexts

pub mod user_id;
pub mod session_id;
pub mod email;
pub mod common_types;
pub mod identifiers;
pub mod payments;

// Re-export commonly used value objects
pub use user_id::UserId;
pub use session_id::{SessionId, SessId};
pub use email::Email;
pub use common_types::*;
pub use identifiers::*;
pub use payments::{Currency, Network};