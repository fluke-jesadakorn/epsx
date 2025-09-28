// Shared entities that are used across multiple bounded contexts

pub mod audit;
pub mod eps_growth;
pub mod stock;
pub mod market_data;
pub mod auth;
pub mod user;

// Re-export common entity types
pub use crate::domain::user_management::aggregates::Session;
pub use audit::*;
pub use eps_growth::*;
pub use stock::*;
pub use market_data::*;
pub use auth::*;
pub use user::*;

// Common entity traits and patterns
pub trait Entity {
    type Id;
    fn id(&self) -> &Self::Id;
}

pub trait AggregateRoot: Entity {
    fn version(&self) -> u64;
}