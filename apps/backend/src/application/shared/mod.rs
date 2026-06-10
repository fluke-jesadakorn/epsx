// Shared Application Layer Concerns
// Common patterns and abstractions used across all application services

pub mod command_bus;
pub mod query_bus;
pub mod validation;
pub mod error;

pub use command_bus::{Command, CommandHandler};
pub use query_bus::{Query, QueryHandler, PaginationParams, SortParams, SortDirection};
pub use validation::{ValidationError, Validator, ValidationUtils};
pub use error::{ApplicationError, ApplicationResult};