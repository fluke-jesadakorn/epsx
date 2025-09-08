// User Management Aggregates
// Aggregates are consistency boundaries that encapsulate business rules and behavior

pub mod user;
pub mod session;

pub use user::User;
pub use session::Session;